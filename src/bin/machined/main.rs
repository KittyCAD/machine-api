use anyhow::Result;
use clap::{Parser, Subcommand};
use machine_api::server;
use std::str::FromStr;
use tokio::sync::RwLock;
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

mod config;
use config::Config;

/// Serve the machine-api server.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "machined")]
#[command(version = "1.0")]
struct Cli {
    /// verbosity of logging output [tracing, debug, info, warn, error]
    #[arg(long, short, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Serve HTTP requests to construct 3D real-world objects from a
    /// specific design.
    Serve {
        /// `host:port` to bind to on the host system.
        #[arg(default_value = "127.0.0.1:8080")]
        bind: String,

        /// Config file to use
        #[arg(default_value = "machine-api.yaml")]
        config: String,
    },
}

async fn main_serve(_cli: &Cli, bind: &str, config: &str) -> Result<()> {
    let cfg: Config = serde_yaml::from_reader(std::fs::File::open(&config)?)?;

    server::serve(
        bind,
        cfg.load()
            .await?
            .into_iter()
            .map(|(machine_id, machine)| (machine_id, RwLock::new(machine)))
            .collect(),
    )
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let subscriber = FmtSubscriber::builder()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::from_str(&cli.log_level).unwrap())
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    match cli.command {
        Commands::Serve { ref bind, ref config } => main_serve(&cli, bind, config).await,
    }
}
