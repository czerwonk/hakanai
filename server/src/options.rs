use clap::Parser;

#[derive(Parser)]
#[command(
    version,
    name = "hakanai-server",
    author = "Daniel Brendgen-Czerwonk",
    about = "A minimalist one-time secret sharing web service. Share sensitive data through ephemeral links that self-destruct after a single view. No accounts, no tracking, just a simple way to transmit secrets that vanish like morning mist."
)]
pub struct Args {
    #[arg(short, long, value_name = "PORT", default_value = "8080")]
    pub port: u16,

    #[arg(
        short,
        long,
        value_name = "LISTEN_ADDRESS",
        default_value = "127.0.0.1"
    )]
    pub listen_address: String,
}
