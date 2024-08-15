use anyhow::Result;
use clap::{Parser, Subcommand};
use machine_api::server;
use opentelemetry::{trace::TracerProvider, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use tokio::sync::RwLock;
use tracing_subscriber::prelude::*;

mod config;
use config::Config;

/// Serve the machine-api server.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "machined")]
#[command(version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Serve HTTP requests to construct 3D real-world objects from a
    /// specific design.
    Serve {
        /// `host:port` to bind to on the host system.
        #[arg(long, short, default_value = "localhost:8080")]
        bind: String,

        /// Config file to use
        #[arg(long, short, default_value = "machine-api.yaml")]
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

    let otlp_host = match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        Ok(val) => val,
        Err(_) => "http://localhost:4317".to_string(),
    };

    // otel uses async runtime, so it must be started after the runtime
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

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
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

    match cli.command {
        Commands::Serve { ref bind, ref config } => main_serve(&cli, bind, config).await,
    }
}
