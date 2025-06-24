use clap::{Parser, Subcommand};
use url::Url;

/// Represents the command-line arguments for the application.
#[derive(Parser)]
#[command(
    version,
    name = "hakanai",
    author = "Daniel Brendgen-Czerwonk",
    about = "A minimalist one-time secret sharing web service. Share sensitive data through ephemeral links that self-destruct after a single view. No accounts, no tracking, just a simple way to transmit secrets that vanish like morning mist."
)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Receives an ephemeral secret from the server.
    Get { link: Url },

    /// Send a secret to the server.
    /// Content is read from stdin.
    Send {
        #[arg(
            short,
            long,
            default_value = "http://localhost:8080",
            help = "Hakanai Server URL to send the secret to (eg. https://hakanai.routing.rocks)."
        )]
        server: Url,
    },
}
