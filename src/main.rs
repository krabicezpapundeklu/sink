use std::{
    env::current_exe,
    ffi::{c_char, CString, OsStr, OsString},
    path::PathBuf,
    process::exit,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use libc::c_int;
use shared::AppContext;
use tracing_subscriber::EnvFilter;

mod repository;
mod server;
mod shared;

extern "C" {
    fn shell_main(argc: c_int, argv: *const *const c_char) -> c_int;
}

#[derive(Parser)]
#[command(disable_help_flag = true, version)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Fix items
    Fix {
        /// Database file to use
        #[arg(default_value = "sink.db", long, short)]
        db: PathBuf,

        /// Only log changes, don't do anything
        #[arg(default_value_t = false, long)]
        dry: bool,
    },
    /// Enter SQL shell
    Shell {
        /// Arguments passed directly to the shell
        args: Vec<OsString>,
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

async fn fix_items(db: PathBuf, dry: bool) -> Result<()> {
    let app_context = AppContext::new(db).await?;

    for id in app_context.get_all_item_ids().await? {
        let item = app_context.get_item(id).await?;
        let old_item_summary = item.summary;
        let mut new_item_summary = old_item_summary.clone();
        new_item_summary.r#type = app_context.get_item_type(&item.body);
        new_item_summary.system = app_context.get_system(&item.headers, &item.body);
        new_item_summary.event_id = AppContext::get_event_id(&item.headers);
        new_item_summary.entity_event_id = app_context.get_entity_event_id(&item.body);

        if new_item_summary != old_item_summary {
            println!("{old_item_summary:?} -> {new_item_summary:?}");

            if !dry {
                app_context.update_item(new_item_summary).await?;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("tower_http::trace=debug,info")),
        )
        .init();

    match args.command {
        Command::Fix { db, dry } => fix_items(db, dry).await,
        Command::Shell { args } => unsafe {
            let mut c_strings = Vec::new();

            c_strings.push(to_cstring(current_exe()?.as_os_str())?);

            for arg in &args {
                c_strings.push(to_cstring(arg)?);
            }

            let c_chars: Vec<*const c_char> = c_strings.iter().map(|cs| cs.as_ptr()).collect();
            let result = shell_main(c_int::try_from(c_chars.len())?, c_chars.as_ptr());

            exit(result);
        },
        Command::StartServer { host, port, db } => server::start(&host, port, db)
            .await
            .context("error running server"),
    }
}

fn to_cstring(os_string: &OsStr) -> Result<CString> {
    CString::new(os_string.to_string_lossy().as_bytes()).map_err(Into::into)
}
