mod cli;
mod factory;
mod get;
mod helper;
mod mock_client;
mod observer;
mod send;

use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::cli::Args;
use crate::get::get;
use crate::send::send;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    if let Err(err) = process_command(args).await {
        eprintln!("{}", err.to_string().red());
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

async fn process_command(args: Args) -> Result<()> {
    let app_factory = factory::AppFactory {};
    match args.command {
        cli::Command::Get {
            link,
            to_stdout,
            filename,
        } => get(app_factory, link, to_stdout, filename).await,
        cli::Command::Send {
            server,
            ttl,
            token,
            file,
            as_file,
            filename,
        } => send(app_factory, server, ttl, token, file, as_file, filename).await,
    }
}
