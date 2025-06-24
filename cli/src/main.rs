mod cli;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::Args;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    ExitCode::SUCCESS
}
