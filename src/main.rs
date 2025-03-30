use std::{
    io::{IsTerminal, stdout},
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;
use sink::{repository::open_repository, server::start, service::new_service};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    /// Host to listen on
    #[arg(default_value = "127.0.0.1", long)]
    host: String,

    /// Port to listen on
    #[arg(default_value_t = 8080, long)]
    port: u16,

    /// Database file to use
    #[arg(default_value = "sink.db", long)]
    db: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let args = Args::parse();

    tracing_subscriber::registry()
        .with(fmt::layer().with_ansi(stdout().is_terminal()))
        .with(EnvFilter::from_default_env())
        .init();

    info!(version = VERSION, ?args, "starting");

    let repository = open_repository(args.db).await?;
    let service = new_service(repository)?;

    start(&args.host, args.port, service).await
}
