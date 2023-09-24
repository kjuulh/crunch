use clap::{Args, Parser, Subcommand, ValueEnum};
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
    crunch_file: String,
}

#[derive(Clone, ValueEnum)]
enum LogArg {
    None,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    init_logging(&cli.global_args.log);

    match &cli.commands {
        Commands::Generate {} => {}
    }
}

fn init_logging(log: &LogArg) {
    match log {
        LogArg::None => {}
        LogArg::Trace => {
            tracing_subscriber::fmt()
                .with_max_level(Level::TRACE)
                .init();
        }
        LogArg::Debug => {
            tracing_subscriber::fmt()
                .with_max_level(Level::DEBUG)
                .init();
        }
        LogArg::Info => {
            tracing_subscriber::fmt().with_max_level(Level::INFO).init();
        }
        LogArg::Warn => {
            tracing_subscriber::fmt().with_max_level(Level::WARN).init();
        }
        LogArg::Error => {
            tracing_subscriber::fmt()
                .with_max_level(Level::ERROR)
                .init();
        }
    }
}
