use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use toml_edit::{value, Document};

#[derive(Debug)]
pub struct File {
    doc: Document,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub service: Service,
    pub publish: Option<Vec<Publish>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Service {
    pub service: String,
    pub domain: String,
    pub codegen: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Publish {
    #[serde(alias = "schema-path")]
    pub schema_path: String,
    #[serde(alias = "output-path")]
    pub output_path: String,
    pub entities: Vec<String>,
}

#[allow(dead_code)]
impl File {
    pub async fn parse_file(path: &std::path::Path) -> anyhow::Result<File> {
        tracing::debug!("loading crunch file at: {}", path.display());

        let file = tokio::fs::read_to_string(path).await?;

        Self::parse(&file).await
    }

    pub async fn parse(content: &str) -> anyhow::Result<File> {
        tracing::debug!("parsing crunch file");
        let config: Document = content.parse::<Document>()?;

        Ok(File { doc: config })
    }

    pub async fn write_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = self.write().await?;

        tracing::debug!("writing to file: {}", path.display());
        let mut file = tokio::fs::File::create(path).await?;

        file.write_all(content.as_bytes()).await?;
        file.sync_all().await?;

        Ok(())
    }

    pub async fn write(&self) -> anyhow::Result<String> {
        tracing::debug!("converting crunch config into file");

        let content = self.doc.to_string();

        Ok(content)
    }

    pub fn add_publish(
        &mut self,
        schema_path: &str,
        output_path: &str,
        entities: &[&str],
    ) -> &mut Self {
        let mut publish = toml_edit::Table::new();
        publish["schema-path"] = value(schema_path);
        publish["output-path"] = value(output_path);

        let mut entities_arr = toml_edit::Array::new();
        for entity in entities {
            entities_arr.push(entity.to_string());
        }
        publish["entities"] = value(entities_arr);

        if !self.doc.contains_key("publish") {
            tracing::debug!("publish key not existing, adding new");
            self.doc["publish"] = toml_edit::array()
        }

        tracing::debug!("adding new publish item");
        self.doc["publish"]
            .as_array_of_tables_mut()
            .expect("publish to be present and be array of tables [[publish]]")
            .push(publish);

        self
    }

    pub fn get_config(&self) -> anyhow::Result<Config> {
        let content = self.doc.to_string();

        tracing::debug!("converting config into read only copy");
        let config: Config = toml_edit::de::from_str(&content)?;

        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_can_write() -> anyhow::Result<()> {
        let raw = r#"[service]
service = "my-service"
domain = "my-domain"
codegen = ["rust"]

[[publish]]	
schema-path = "schemas/crunch"
output-path = "src/crunch"
"#;
        let output = File::parse(raw).await?.write().await?;

        pretty_assertions::assert_eq!(output, raw);

        Ok(())
    }

    #[tokio::test]
    async fn test_can_add_publish() -> anyhow::Result<()> {
        let raw = r#"[service]
service = "my-service"
domain = "my-domain"
codegen = ["rust"]

[[publish]]	
schema-path = "schemas/crunch"
output-path = "src/crunch"
"#;
        let expected = r#"[service]
service = "my-service"
domain = "my-domain"
codegen = ["rust"]

[[publish]]	
schema-path = "schemas/crunch"
output-path = "src/crunch"

[[publish]]
schema-path = "some-schema"
output-path = "some-output"
"#;
        let mut config = File::parse(raw).await?;
        let config = config.add_publish("some-schema", "some-output", &[]);
        let output = config.write().await?;

        pretty_assertions::assert_eq!(output, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_can_add_publish_if_none() -> anyhow::Result<()> {
        let raw = r#"[service]
service = "my-service"
domain = "my-domain"
codegen = ["rust"]
"#;
        let expected = r#"[service]
service = "my-service"
domain = "my-domain"
codegen = ["rust"]

[[publish]]
schema-path = "some-schema"
output-path = "some-output"
"#;
        let mut config = File::parse(raw).await?;
        let config = config.add_publish("some-schema", "some-output", &[]);
        let output = config.write().await?;

        pretty_assertions::assert_eq!(output, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_can_get_config() -> anyhow::Result<()> {
        let raw = r#"[service]
service = "my-service"
domain = "my-domain"
codegen = ["rust"]
"#;

        let config = File::parse(raw).await?.get_config()?;

        pretty_assertions::assert_eq!(
            config,
            Config {
                service: Service {
                    service: "my-service".into(),
                    domain: "my-domain".into(),
                    codegen: vec!["rust".into()]
                },
                publish: None
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_can_get_config_publish() -> anyhow::Result<()> {
        let raw = r#"[service]
service = "my-service"
domain = "my-domain"
codegen = ["rust"]

[[publish]]
schema-path = "some-schema"
output-path = "some-output"
"#;

        let config = File::parse(raw).await?.get_config()?;

        pretty_assertions::assert_eq!(
            config,
            Config {
                service: Service {
                    service: "my-service".into(),
                    domain: "my-domain".into(),
                    codegen: vec!["rust".into()]
                },
                publish: Some(vec![Publish {
                    schema_path: "some-schema".into(),
                    output_path: "some-output".into()
                }])
            }
        );

        Ok(())
    }
}
