use anyhow::anyhow;
use genco::prelude::*;
use regex::Regex;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;

#[derive(Debug)]
struct Node {
    file: Option<String>,
    messages: Option<Vec<String>>,
    segment: String,
    children: HashMap<String, Node>,
}

impl Node {
    fn new(segment: String, file: Option<String>, messages: Option<Vec<String>>) -> Self {
        Node {
            file,
            messages,
            segment,
            children: HashMap::new(),
        }
    }

    fn insert(&mut self, file_name: &str, messages: Vec<String>) {
        let mut node = self;
        let file_name_content = PathBuf::from(file_name);
        let file_name_content = file_name_content.file_stem().unwrap();
        let file_name_content = file_name_content.to_string_lossy().to_lowercase();

        let segments = file_name_content.split('.').collect::<Vec<_>>();
        for (i, segment) in segments.iter().enumerate() {
            node = node.children.entry(segment.to_string()).or_insert_with(|| {
                Node::new(
                    segment.to_string(),
                    if i + 1 == segments.len() {
                        Some(file_name.into())
                    } else {
                        None
                    },
                    if i + 1 == segments.len() {
                        Some(messages.clone())
                    } else {
                        None
                    },
                )
            });
        }
    }

    fn traverse(&self) -> genco::lang::rust::Tokens {
        let mut child_tokens = Vec::new();
        let mut nodes = self.children.iter().map(|(_, n)| n).collect::<Vec<_>>();
        nodes.sort_by(|a, b| a.segment.cmp(&b.segment));
        for node in nodes {
            let tokens = node.traverse_indent(0);
            child_tokens.push(tokens);
        }

        quote! {
            pub mod $(&self.segment) {
                $(for tokens in child_tokens join ($['\r']) => $tokens)
            }
        }
    }

    fn traverse_indent(&self, indent: usize) -> genco::lang::rust::Tokens {
        tracing::trace!("node traverse visited: {}", self.segment);

        let mut message_tokens = Vec::new();
        if let Some(file) = &self.file {
            if let Some(messages) = &self.messages {
                for message in messages.iter() {
                    tracing::trace!("node traverse visited message: {}", message);
                    let tokens: genco::lang::rust::Tokens = quote! {
                    impl ::crunch::traits::Serializer for $(message) {
                        fn serialize(&self) -> Result<Vec<u8>, ::crunch::errors::SerializeError> {
                            Ok(self.encode_to_vec())
                        }
                    }
                    impl ::crunch::traits::Deserializer for $(message) {
                        fn deserialize(raw: Vec<u8>) -> Result<Self, ::crunch::errors::DeserializeError>
                        where
                            Self: Sized,
                        {
                            let output  = Self::decode(raw.as_slice()).map_err(|e| ::crunch::errors::DeserializeError::ProtoErr(e))?;
                            Ok(output)
                        }
                    }

                    impl crunch::traits::Event for $(message) {
                        fn event_info() -> ::crunch::traits::EventInfo {
                            ::crunch::traits::EventInfo {
                                domain: "my-domain",
                                entity_type: "my-entity-type",
                                event_name: "my-event-name",
                            }
                        }
                    }
                    };

                    message_tokens.push(tokens);
                }
            }

            quote! {
                pub mod $(&self.segment) {
                    use prost::Message;
                    include!($(quoted(file)));
                    $(for tokens in message_tokens join ($['\r']) => $tokens)
                }
            }
        } else {
            let mut child_tokens = Vec::new();
            let mut nodes = self.children.iter().map(|(_, n)| n).collect::<Vec<_>>();
            nodes.sort_by(|a, b| a.segment.cmp(&b.segment));
            for node in nodes {
                let tokens = node.traverse_indent(indent + 1);
                child_tokens.push(tokens);
            }

            quote! {
                pub mod $(&self.segment) {
                    $(for tokens in child_tokens join ($['\r']) => $tokens)
                }
            }
        }
    }
}

pub struct Codegen {}

impl Codegen {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn generate_rust(&self, input_path: &Path, output_path: &Path) -> anyhow::Result<()> {
        if output_path.exists() {
            tokio::fs::remove_dir_all(output_path).await?;
        }

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

        let mut output_paths = self.discover_files(out_tempdir_path, "rs")?;

        let mod_path = self
            .generate_mod_file(out_tempdir_path, &output_paths)
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
        let mut node = Node::new("root".into(), None, None);

        let regex = Regex::new(r"pub struct (?P<eventName>[a-zA-Z0-9-_]+)")
            .expect("regex to be well formed");

        let mut output_paths = output_paths.to_vec();
        output_paths.sort();

        for generated_file in output_paths {
            if let Some(name) = generated_file.file_name() {
                let file_name = name.to_str().unwrap();
                let file = tokio::fs::read_to_string(&generated_file).await?;
                let mut messages = regex
                    .captures_iter(&file)
                    .map(|m| m.name("eventName").unwrap())
                    .map(|m| m.as_str().to_string())
                    .collect::<Vec<_>>();
                messages.sort();

                node.insert(file_name, messages);
            }
        }
        let mod_tokens: genco::lang::rust::Tokens = genco::quote! {
            $(node.traverse())
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
    use super::*;
    #[test]
    fn test_node() {
        let mut root = Node::new("root".into(), None, None);

        root.insert("basic.my_event.rs", vec!["One".into(), "Two".into()]);
        root.insert("basic.includes.includes.rs", vec!["Three".into()]);
        root.insert("basic.includes.includes-two.rs", Vec::new());

        let res = root
            .traverse()
            .to_file_string()
            .expect("to generate rust code");

        pretty_assertions::assert_eq!(res, r#""#);

        panic!();
    }
}
