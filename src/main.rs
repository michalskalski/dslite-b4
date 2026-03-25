use dslite_b4::config::Config;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dslite-b4", about = "DS-Lite B4 client")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(short, long)]
        config: PathBuf,
    },
    CheckConfig {
        #[arg(short, long)]
        config: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dslite_b4=info".parse().unwrap()),
        )
        .init();
    let cli = Cli::parse();
    match cli.command {
        Commands::CheckConfig { config } => {
            let config = toml::from_str::<Config>(&std::fs::read_to_string(config)?)?;
            tracing::info!(?config);
        }
        Commands::Run { config: _ } => {
            todo!()
        }
    }
    Ok(())
}
