use std::net::SocketAddr;

use axum::routing::get;
use axum::Router;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None, subcommand_required = true)]
struct Command {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(env = "SERVICE_HOST", long, default_value = "127.0.0.1:3000")]
        host: SocketAddr,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cli = Command::parse();

    match cli.command {
        Some(Commands::Serve { host }) => {
            tracing::info!("Starting service");

            let app = Router::new().route("/", get(root));

            tracing::info!("listening on {}", host);
            axum::Server::bind(&host)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        None => {}
    }

    Ok(())
}

async fn root() -> &'static str {
    "Hello, crunch!"
}
