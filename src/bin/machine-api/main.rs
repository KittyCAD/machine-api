use anyhow::Result;
use clap::{Parser, Subcommand};
use opentelemetry::{trace::TracerProvider, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use tracing_subscriber::prelude::*;

mod config;
use config::Config;

mod cmd_serve;

/// Serve the machine-api server.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "machine-api")]
#[command(version = "1.0")]
struct Cli {
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

    let level_filter = tracing_subscriber::filter::LevelFilter::DEBUG;

    let otlp_host = match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        Ok(val) => val,
        Err(_) => "http://localhost:4317".to_string(),
    };

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_endpoint(otlp_host))
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_resource(Resource::new(vec![KeyValue::new("service.name", "machine-api")])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    opentelemetry::global::set_tracer_provider(provider.clone());
    let tracer = provider.tracer("tracing-otel-subscriber");

    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(level_filter);

    // Initialize tracing.
    tracing_subscriber::registry()
        // .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::Layer::default())
        .with(telemetry)
        .with({
            #[cfg(feature = "debug")]
            {
                // When running with `debug`, we're going to hook in the console
                // subscriber for tokio-console.
                console_subscriber::spawn()
            }
            #[cfg(not(feature = "debug"))]
            {
                // Under normal cases, we need a blank Layer that doesn't
                // do anything.
                tracing_subscriber::layer::Identity::new()
            }
        })
        .init();

    #[cfg(feature = "debug")]
    {
        delouse::init()?;
    }

    let cfg: Config = toml::from_str(
        &std::fs::read_to_string(&cli.config)
            .map_err(|_| anyhow::anyhow!("Config file not found at {}", &cli.config))?,
    )?;

    match cli.command {
        Commands::Serve { ref bind } => cmd_serve::main(&cli, &cfg, bind).await,
    }
}
