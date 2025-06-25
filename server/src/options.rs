use clap::Parser;

/// Represents the command-line arguments for the server.
#[derive(Parser)]
#[command(
    version,
    name = "hakanai-server",
    author = "Daniel Brendgen-Czerwonk",
    about = "A minimalist one-time secret sharing web service. Share sensitive data through ephemeral links that self-destruct after a single view. No accounts, no tracking, just a simple way to transmit secrets that vanish like morning mist."
)]
pub struct Args {
    /// The port on which the server will listen for incoming connections.
    #[arg(
        short,
        long,
        value_name = "PORT",
        env = "HAKANAI_PORT",
        default_value = "8080"
    )]
    pub port: u16,

    /// The network address on which the server will listen.
    #[arg(
        short,
        long,
        value_name = "LISTEN_ADDRESS",
        env = "HAKANAI_LISTEN_ADDRESS",
        default_value = "127.0.0.1"
    )]
    pub listen_address: String,

    /// The Data Source Name (DSN) for the Redis database.
    #[arg(
        short,
        long,
        value_name = "REDIS_DSN",
        env = "HAKANAI_REDIS_DSN",
        default_value = "redis://127.0.0.1:6379/"
    )]
    pub redis_dsn: String,

    /// List of tokens allowed to create new secrets. If empty, no tokens are required.
    #[arg(
        short,
        long,
        value_name = "TOKENS",
        env = "HAKANAI_TOKENS",
        default_value = ""
    )]
    pub tokens: Vec<String>,
}
