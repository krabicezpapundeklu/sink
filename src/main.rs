use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use sink::{repository::open_repository, server::start, service::new_service};

#[derive(Parser)]
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
    let args = Args::parse();

    let repository = open_repository(args.db).await?;
    let service = new_service(repository)?;

    start(&args.host, args.port, service).await
}
