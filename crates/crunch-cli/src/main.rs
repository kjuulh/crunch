mod logging;

use std::path::PathBuf;

use anyhow::anyhow;
use clap::{Args, Parser, Subcommand, ValueEnum};
use logging::LogArg;
use tracing::Level;

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
            let config_file = config::get_file(&cli.global_args.crunch_file)
                .await
                .map_err(|e| anyhow!("failed to load config: {}", e))?
                .get_config()
                .map_err(|e| anyhow!("invalid config: {}", e))?;

            tracing::info!("generating crunch code")
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
