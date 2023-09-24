mod logging;

use std::path::{PathBuf};

use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use logging::LogArg;

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
    }

    Ok(())
}

mod config {
    pub async fn get_file(path: &std::path::Path) -> anyhow::Result<crunch_file::File> {
        let file = crunch_file::File::parse_file(path).await?;

        Ok(file)
    }
}
