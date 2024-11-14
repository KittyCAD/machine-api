use anyhow::Result;
use clap::{Parser, Subcommand};
use opentelemetry::trace::TracerProvider;
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

    /// Print debug info
    #[clap(short, long)]
    pub debug: bool,

    /// Print logs as json
    #[clap(short, long)]
    pub json: bool,
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

    let level_filter = if cli.debug {
        tracing_subscriber::filter::LevelFilter::DEBUG
    } else {
        tracing_subscriber::filter::LevelFilter::INFO
    };

    // Format fields using the provided closure.
    // We want to make this very consise otherwise the logs are not able to be read by humans.
    let format = tracing_subscriber::fmt::format::debug_fn(|writer, field, value| {
        if format!("{field}") == "message" {
            write!(writer, "{field}: {value:?}")
        } else {
            write!(writer, "{field}")
        }
    })
    // Separate each field with a comma.
    // This method is provided by an extension trait in the
    // `tracing-subscriber` prelude.
    .delimited(", ");

    let (json, plain) = if cli.json {
        // Cloud run likes json formatted logs if possible.
        // See: https://cloud.google.com/run/docs/logging
        // We could probably format these specifically for cloud run if we wanted,
        // will save that as a TODO: https://cloud.google.com/run/docs/logging#special-fields
        (
            Some(tracing_subscriber::fmt::layer().json().with_filter(level_filter)),
            None,
        )
    } else {
        (
            None,
            Some(
                tracing_subscriber::fmt::layer()
                    .pretty()
                    .fmt_fields(format)
                    .with_filter(level_filter),
            ),
        )
    };

    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::SpanExporter::builder().with_tonic().build()?,
            opentelemetry_sdk::runtime::Tokio,
        )
        .build();

    opentelemetry::global::set_tracer_provider(provider.clone());
    let tracer = provider.tracer("tracing-otel-subscriber");

    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(level_filter);

    // Initialize tracing.
    tracing_subscriber::registry()
        .with(json)
        .with(plain)
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
