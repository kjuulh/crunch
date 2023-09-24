use anyhow::anyhow;
use genco::prelude::*;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;

pub struct Codegen {}

impl Codegen {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn generate_rust(&self, input_path: &Path, output_path: &Path) -> anyhow::Result<()> {
        let input_protos = self.discover_files(input_path, "proto")?;
        let (input_proto_paths, input_dir) = self.copy_protos(input_protos, input_path).await?;
        let (output_proto_paths, temp_output_dir) = self
            .generate_rust_from_proto(input_proto_paths, input_dir.path())
            .await?;

        self.copy_rs(output_proto_paths, output_path, temp_output_dir.path())
            .await?;

        Ok(())
    }

    fn discover_files(&self, input_path: &Path, extension: &str) -> anyhow::Result<Vec<PathBuf>> {
        let mut proto_files = Vec::new();
        for entry in WalkDir::new(input_path) {
            let entry = entry?;

            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if ext == extension {
                    proto_files.push(entry.into_path());
                }
            }
        }

        if proto_files.is_empty() {
            anyhow::bail!(
                "failed to find any protobuf files in: {}",
                input_path.display()
            );
        }

        Ok(proto_files)
    }

    async fn copy_protos(
        &self,
        input_protos: Vec<PathBuf>,
        root_path: &Path,
    ) -> anyhow::Result<(Vec<PathBuf>, tempfile::TempDir)> {
        let in_tempdir = tempfile::TempDir::new()?;
        let in_tempdir_path = in_tempdir.path();

        let mut input_proto_paths = Vec::new();
        for input_proto in &input_protos {
            let rel_proto_path = input_proto.strip_prefix(root_path)?;
            let in_proto_path = in_tempdir_path.join(rel_proto_path);
            if let Some(dir) = in_proto_path.parent() {
                if !dir.exists() {
                    tokio::fs::create_dir_all(dir).await?;
                }
            }

            tokio::fs::copy(input_proto, &in_proto_path).await?;
            input_proto_paths.push(in_proto_path);
        }

        Ok((input_proto_paths, in_tempdir))
    }

    async fn generate_rust_from_proto(
        &self,
        input_proto_paths: Vec<PathBuf>,
        in_root_path: &Path,
    ) -> anyhow::Result<(Vec<PathBuf>, tempfile::TempDir)> {
        let out_tempdir = tempfile::TempDir::new()?;
        let out_tempdir_path = out_tempdir.path();
        let handle = tokio::task::spawn_blocking({
            let out_tempdir_path = out_tempdir_path.to_path_buf();
            let in_root_path = in_root_path.to_path_buf();
            move || {
                prost_build::Config::new()
                    .out_dir(out_tempdir_path)
                    .compile_protos(input_proto_paths.as_slice(), &[in_root_path])?;

                Ok(())
            }
        });

        let result: anyhow::Result<()> = handle.await?;
        result?;

        let mut output_paths = self.discover_files(&out_tempdir_path, "rs")?;

        let mod_path = self
            .generate_mod_file(&out_tempdir_path, &output_paths)
            .await?;
        output_paths.push(mod_path);

        Ok((output_paths, out_tempdir))
    }

    async fn generate_mod_file(
        &self,
        output_tempdir_path: &Path,
        output_paths: &[PathBuf],
    ) -> anyhow::Result<PathBuf> {
        let mod_path = output_tempdir_path.join("mod.rs");
        let mut mod_file = tokio::fs::File::create(&mod_path).await?;

        let mut includes: Vec<genco::lang::rust::Tokens> = Vec::new();
        for generated_file in output_paths {
            if let Some(name) = generated_file.file_name() {
                let mod_name = generated_file
                    .file_stem()
                    .unwrap()
                    .to_ascii_lowercase()
                    .to_string_lossy()
                    .replace(".", "_")
                    .replace("-", "_");

                let file_name = name.to_str().unwrap();

                includes.push(genco::quote! {
                    pub mod $(mod_name) {
                        include!($(quoted(file_name)));
                    }
                });
            }
        }

        let mod_tokens: genco::lang::rust::Tokens = genco::quote! {
            $(for tokens in includes join($['\n']) => $tokens)
        };
        let mod_contents = mod_tokens.to_file_string()?;
        mod_file.write_all(mod_contents.as_bytes()).await?;

        Ok(mod_path)
    }

    async fn copy_rs(
        &self,
        output_proto_paths: Vec<PathBuf>,
        output_path: &Path,
        root_path: &Path,
    ) -> anyhow::Result<()> {
        for output_rs in &output_proto_paths {
            let rel_proto_path = output_rs.strip_prefix(root_path).map_err(|e| {
                anyhow!(
                    "output: {} does not match root_path: {}, error: {}",
                    output_rs.display(),
                    root_path.display(),
                    e
                )
            })?;
            let in_proto_path = output_path.join(rel_proto_path);
            if let Some(dir) = in_proto_path.parent() {
                if !dir.exists() {
                    tokio::fs::create_dir_all(dir).await?;
                }
            }

            tokio::fs::copy(output_rs, &in_proto_path).await?;
        }

        Ok(())
    }
}

impl Default for Codegen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use genco::prelude::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_can_generate_output_rust() -> anyhow::Result<()> {
        // Generate from protobuf
        let proto_spec = r#"
syntax = "proto3";

import "includes/test_include.proto";

package test.can.generate.output.rust;

message MyEvent {
    string name = 1;
}
"#;

        let proto_include_spec = r#"
syntax = "proto3";

package test.can.generate.output.rust.include.test_include;

message MyInclude {
    string name = 1;
}
"#;

        let out_tempdir = tempfile::TempDir::new()?;
        let in_tempdir = tempfile::TempDir::new()?;

        let proto_path = in_tempdir.path().join("test.proto");
        let mut proto_file = tokio::fs::File::create(&proto_path).await?;
        proto_file.write_all(proto_spec.as_bytes()).await?;
        proto_file.sync_all().await?;

        tokio::fs::create_dir_all(in_tempdir.path().join("includes")).await?;
        let proto_include_path = in_tempdir.path().join("includes/test_include.proto");
        let mut proto_file = tokio::fs::File::create(&proto_include_path).await?;
        proto_file.write_all(proto_include_spec.as_bytes()).await?;
        proto_file.sync_all().await?;

        let out_tempdir_path = out_tempdir.into_path();
        let handle = tokio::task::spawn_blocking({
            let out_tempdir_path = out_tempdir_path.clone();
            move || {
                prost_build::Config::new()
                    .out_dir(out_tempdir_path)
                    .compile_protos(&[proto_path, proto_include_path], &[in_tempdir.into_path()])?;

                Ok(())
            }
        });

        let result: anyhow::Result<()> = handle.await?;
        result?;

        let mut entries = tokio::fs::read_dir(&out_tempdir_path).await?;
        let mut file_paths = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if ext == "rs" {
                    file_paths.push(entry.path());
                }
            }
        }

        // Generate mod.rs
        let mod_path = out_tempdir_path.join("mod.rs");
        let mut mod_file = tokio::fs::File::create(&mod_path).await?;

        let mut includes: Vec<genco::lang::rust::Tokens> = Vec::new();
        for generated_file in &file_paths {
            if let Some(name) = generated_file.file_name() {
                let mod_name = generated_file
                    .file_stem()
                    .unwrap()
                    .to_ascii_lowercase()
                    .to_string_lossy()
                    .replace(".", "_")
                    .replace("-", "_");

                let file_name = name.to_str().unwrap();

                includes.push(genco::quote! {
                    pub mod $(mod_name) {
                        include!($(quoted(file_name)));
                    }
                });
            }
        }

        let mod_tokens: genco::lang::rust::Tokens = genco::quote! {
            $(for tokens in includes join($['\n']) => $tokens)
        };
        let mod_contents = mod_tokens.to_file_string()?;

        pretty_assertions::assert_eq!("", mod_contents);

        mod_file.write_all(mod_contents.as_bytes()).await?;

        assert_eq!(1, file_paths.len());

        Ok(())
    }
}
