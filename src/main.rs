#[cfg(target_os = "linux")]
use dslite_b4::tunnel::linux::LinuxBackend;
use dslite_b4::{config::Config, dns::resolve, tunnel::TunnelBackend};
use std::path::PathBuf;
use tokio::signal;

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
async fn main() -> anyhow::Result<()> {
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
        Commands::Run { config } => {
            let config = toml::from_str::<Config>(&std::fs::read_to_string(config)?)?;
            let aftr_ip = resolve(&config.aftr.address).await?;
            #[cfg(target_os = "linux")]
            let backend = LinuxBackend::new(
                config.tunnel.name,
                config.tunnel.local_v6,
                aftr_ip,
                config.tunnel.local_v4,
            );

            run_daemon(backend).await?
        }
    }
    Ok(())
}

async fn run_daemon<B: TunnelBackend>(backend: B) -> anyhow::Result<()> {
    backend.setup().await?;

    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;
    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = sigterm.recv() => {},
    };

    backend.teardown().await?;
    Ok(())
}
