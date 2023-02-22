use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use env_logger::Env;
use import::import_csv;
use server::start_server;

mod import;
mod repository;
mod server;
mod shared;

#[derive(Parser)]
#[command(disable_help_flag = true, version)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Import items from CSV file
    ImportCsv {
        /// CSV file with items to import
        #[arg(long, short)]
        input: PathBuf,

        /// Database file where items will be written to
        #[arg(default_value = "sink.db", long, short)]
        output: PathBuf,

        /// Generate new IDs for items
        #[arg(long, short)]
        generate_ids: bool,
    },
    /// Start HTTP server listening for new items
    StartServer {
        /// Host to listen on
        #[arg(default_value = "127.0.0.1", long, short)]
        host: String,

        /// Port to listen on
        #[arg(default_value_t = 8080, long, short)]
        port: u16,

        /// Database file to use
        #[arg(default_value = "sink.db", long, short)]
        db: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    match &args.command {
        Command::ImportCsv {
            input,
            output,
            generate_ids,
        } => import_csv(input, output, *generate_ids).context("cannot import items from CSV"),
        Command::StartServer { host, port, db } => {
            start_server(host, *port, db).context("error running server")
        }
    }
}
