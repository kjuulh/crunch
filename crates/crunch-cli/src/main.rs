mod logging;

use std::path::PathBuf;

use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use inquire::validator::Validation;
use logging::LogArg;
use regex::Regex;
use tokio::io::AsyncWriteExt;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None, subcommand_required = true)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,

    #[command(flatten)]
    global_args: GlobalArgs,
}

#[derive(Subcommand, Clone)]
enum Commands {
    Generate {},
    Init {
        #[command(subcommand)]
        commands: Option<InitCommands>,
    },
}

#[derive(Subcommand, Clone)]
enum InitCommands {
    Publish {},
}

#[derive(Args, Clone)]
struct GlobalArgs {
    #[arg(long, default_value = "none", global = true, help_heading = "Global")]
    log: LogArg,

    #[arg(
        long,
        default_value = ".crunch.toml",
        global = true,
        help_heading = "Global"
    )]
    crunch_file: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.global_args.log.init_logging();

    match &cli.commands {
        Commands::Generate {} => {
            let config = config::get_file(&cli.global_args.crunch_file)
                .await
                .map_err(|e| anyhow!("failed to load config: {}", e))?
                .get_config()
                .map_err(|e| anyhow!("invalid config: {}", e))?;

            tracing::info!("generating crunch code");
            let codegen = crunch_codegen::Codegen::new();

            if let Some(publish) = config.publish {
                for p in &publish {
                    let mut rel_schema_path = PathBuf::from(&p.schema_path);
                    let mut rel_output_path = PathBuf::from(&p.output_path);

                    if let Some(dir_path) = cli.global_args.crunch_file.parent() {
                        rel_schema_path = dir_path.join(rel_schema_path);
                        rel_output_path = dir_path.join(rel_output_path);
                    }

                    codegen
                        .generate_rust(&rel_schema_path, &rel_output_path)
                        .await?;

                    println!("success: generated crunch {}", &rel_output_path.display());
                }
            }
        }
        Commands::Init {
            commands: Some(commands),
        } => match commands {
            InitCommands::Publish {} => {
                match config::get_file(&cli.global_args.crunch_file).await {
                    Err(_) => {
                        anyhow::bail!(
                            "config file not found: {}",
                            &cli.global_args
                                .crunch_file
                                .canonicalize()
                                .unwrap_or(cli.global_args.crunch_file)
                                .display()
                        )
                    }
                    Ok(mut config) => {
                        let schema_path = inquire::Text::new("schema_path")
                            .with_help_message(
                                "please select where you want your schema files to be placed",
                            )
                            .with_default("schemas/crunch")
                            .prompt()?;
                        let output_path = inquire::Text::new("output_path")
                            .with_help_message(
                                "please select where you want your generated files to be placed",
                            )
                            .with_default("src/gencrunch")
                            .prompt()?;

                        let config = config.add_publish(&schema_path, &output_path);
                        config.write_file(&cli.global_args.crunch_file).await?;

                        // TODO: add example schema

                        let output_path = if let Some(dir) = &cli.global_args.crunch_file.parent() {
                            if dir.display().to_string() == "" {
                                format!("{schema_path}")
                            } else {
                                format!(
                                    "{}/{}",
                                    dir.display().to_string().trim_end_matches("/"),
                                    schema_path.trim_start_matches("/")
                                )
                            }
                        } else {
                            format!("{schema_path}")
                        };

                        println!("Success: added publish, check schema at: {output_path}");
                    }
                }
            }
        },
        Commands::Init { commands: None } => {
            match config::get_file(&cli.global_args.crunch_file).await {
                Ok(_) => anyhow::bail!("config file already exists"),
                Err(_) => {}
            }

            let path = &cli.global_args.crunch_file;
            let file_name = path
                .file_name()
                .ok_or(anyhow::anyhow!("not a valid file name"))?;
            if file_name != ".crunch.toml" {
                anyhow::bail!("--crunch-file always has to end with file: .crunch.toml");
            }
            if let Some(dir) = path.parent() {
                if !dir.exists() {
                    tokio::fs::create_dir_all(dir).await?;
                }
            }

            fn validate_text(
                text: &str,
            ) -> Result<Validation, Box<dyn std::error::Error + Send + Sync + 'static>>
            {
                let regex = Regex::new("^[a-z0-9-_]+$").expect("is required to be valid regex");
                if regex.is_match(text) {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid(
                        "a service name can only contain lowercase letter, numbers, - and _".into(),
                    ))
                }
            }

            let service = inquire::Text::new("service")
                .with_help_message("please insert your service name")
                .with_validator(validate_text)
                .prompt()?;
            let domain = inquire::Text::new("domain")
                .with_help_message("please insert your domain")
                .with_validator(validate_text)
                .prompt()?;
            let codegen = inquire::MultiSelect::new("codegen", vec!["rust", "go"])
                .with_help_message("which types of client code should be generated for you?")
                .prompt()?;

            let mut crunch_file = tokio::fs::File::create(path).await?;
            crunch_file
                .write_all(
                    format!(
                        r#"[service]
service = "{service}"
domain = "{domain}"
codegen = [{}]
"#,
                        codegen
                            .iter()
                            .map(|c| format!(r#""{c}""#))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                    .as_bytes(),
                )
                .await?;

            println!("Success! generated file at: {}", path.display());
        }
    }

    Ok(())
}

mod config {
    pub async fn get_file(path: &std::path::Path) -> anyhow::Result<crunch_file::File> {
        let file = crunch_file::File::parse_file(path).await?;

        Ok(file)
    }
}
