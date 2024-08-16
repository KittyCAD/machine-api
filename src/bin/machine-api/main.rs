use anyhow::Result;
use clap::{Parser, Subcommand};
use std::str::FromStr;
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

mod config;
use config::Config;

mod cmd_discover;
mod cmd_serve;

/// Serve the machine-api server.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "machine-api")]
#[command(version = "1.0")]
struct Cli {
    /// verbosity of logging output [tracing, debug, info, warn, error]
    #[arg(long, short, default_value = "info")]
    log_level: String,

    /// Config file to use
    #[arg(long, short, default_value = "machine-api.toml")]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Serve HTTP requests to construct 3D real-world objects from a
    /// specific design.
    Serve {
        /// `host:port` to bind to on the host system.
        #[arg(long, short, default_value = "127.0.0.1:8080")]
        bind: String,
    },

    /// Enumerate all visable Machines, connected or otherwise.
    Discover {},
}

async fn handle_signals() -> Result<()> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigint = signal(SignalKind::interrupt()).map_err(|e| {
            tracing::error!(error = format!("{:?}", e), "Failed to set up SIGINT handler");
            e
        })?;
        let mut sigterm = signal(SignalKind::terminate()).map_err(|e| {
            tracing::error!(error = format!("{:?}", e), "Failed to set up SIGTERM handler");
            e
        })?;

        tokio::select! {
            _ = sigint.recv() => {
                tracing::info!("received SIGINT");
            }
            _ = sigterm.recv() => {
                tracing::info!("received SIGTERM");
            }
        }
    }

    #[cfg(windows)]
    {
        tokio::signal::ctrl_c().await.map_err(|e| {
            tracing::error!(error = format!("{:?}", e), "Failed to set up Ctrl+C handler");
            anyhow::Error::new(e)
        })?;

        tracing::info!("received Ctrl+C (SIGINT)");
    }

    tracing::info!("triggering cleanup...");
    tracing::info!("all clean, exiting!");
    std::process::exit(0);
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tokio::spawn(async { handle_signals().await });

    let subscriber = FmtSubscriber::builder()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::from_str(&cli.log_level).unwrap())
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cfg: Config = toml::from_str(
        &std::fs::read_to_string(&cli.config)
            .map_err(|_| anyhow::anyhow!("Config file not found at {}", &cli.config))?,
    )?;

    match cli.command {
        Commands::Serve { ref bind } => cmd_serve::main(&cli, &cfg, bind).await,
        Commands::Discover {} => cmd_discover::main(&cli, &cfg).await,
    }
}
