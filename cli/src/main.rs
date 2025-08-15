// SPDX-License-Identifier: Apache-2.0

mod cli;
mod factory;
mod factory_mock;
mod get;
mod helper;
mod observer;
mod send;
mod token;

use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::cli::Args;
use crate::get::get;
use crate::send::send;
use crate::token::token;

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
        cli::Command::Get(get_args) => get(app_factory, get_args).await,
        cli::Command::Send(send_args) => send(app_factory, send_args).await,
        cli::Command::Token(token_args) => token(token_args).await,
    }
}
