use std::{
    env::current_exe,
    ffi::{c_char, CString, OsString},
    os::unix::prelude::OsStrExt,
    path::PathBuf,
    process::exit,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use env_logger::Env;
use libc::c_int;
use server::start_server;

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

fn main() -> Result<()> {
    let args = Args::parse();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    match &args.command {
        Command::Shell { args } => unsafe {
            let mut c_strings = Vec::new();

            c_strings.push(CString::new(current_exe()?.as_os_str().as_bytes())?);

            for arg in args {
                c_strings.push(CString::new(arg.as_bytes())?);
            }

            let c_chars: Vec<*const c_char> = c_strings.iter().map(|cs| cs.as_ptr()).collect();
            let result = shell_main(c_chars.len() as c_int, c_chars.as_ptr());

            exit(result);
        },
        Command::StartServer { host, port, db } => {
            start_server(host, *port, db).context("error running server")
        }
    }
}
