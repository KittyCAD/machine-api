//! machine-api

#![deny(missing_docs)]

mod gcode;
mod server;
#[cfg(test)]
mod tests;
mod usb_printer;

use std::ffi::OsStr;

use anyhow::{bail, Result};
use clap::Parser;
use gcode::GcodeSequence;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use slog::Drain;
use tracing_subscriber::{prelude::*, Layer};
use usb_printer::UsbPrinter;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields.
#[derive(Parser, Debug, Clone)]
#[clap(version = clap::crate_version!(), author = clap::crate_authors!("\n"))]
pub struct Opts {
    /// Print debug info
    #[clap(short, long)]
    pub debug: bool,

    /// Print logs as json
    #[clap(short, long)]
    pub json: bool,

    /// The subcommand to run.
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

impl Opts {
    /// Setup our logger.
    pub fn create_logger(&self, app: &str) -> slog::Logger {
        if self.json {
            let drain = slog_json::Json::default(std::io::stderr()).fuse();
            self.async_root_logger(drain, app)
        } else {
            let decorator = slog_term::TermDecorator::new().build();
            let drain = slog_term::FullFormat::new(decorator).build().fuse();
            self.async_root_logger(drain, app)
        }
    }

    fn async_root_logger<T>(&self, drain: T, app: &str) -> slog::Logger
    where
        T: slog::Drain + Send + 'static,
        <T as slog::Drain>::Err: std::fmt::Debug,
    {
        let level = if self.debug {
            slog::Level::Debug
        } else {
            slog::Level::Info
        };

        let level_drain = slog::LevelFilter(drain, level).fuse();
        let async_drain = slog_async::Async::new(level_drain).build().fuse();
        slog::Logger::root(async_drain, slog::slog_o!("app" => app.to_owned()))
    }
}

/// A subcommand for our cli.
#[derive(Parser, Debug, Clone)]
pub enum SubCommand {
    /// Run the server.
    Server(Server),

    /// List all available USB devices.
    ListUsbDevices,

    /// Slice the given `file` with config from `config_file`
    SliceFile {
        /// Path to config file for slice
        config_file: std::path::PathBuf,

        /// File path to slice
        file: std::path::PathBuf,
    },

    /// Print the given `file` with config from `config_file`
    PrintFile {
        /// Path to config file for slice
        config_file: std::path::PathBuf,

        /// File path to slice
        file: std::path::PathBuf,
    },
}

/// A subcommand for running the server.
#[derive(Parser, Clone, Debug)]
pub struct Server {
    /// IP address and port that the server should listen
    #[clap(short, long, default_value = "0.0.0.0:8080")]
    pub address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let level_filter = if opts.debug {
        tracing_subscriber::filter::LevelFilter::DEBUG
    } else {
        tracing_subscriber::filter::LevelFilter::INFO
    };

    // Format fields using the provided closure.
    // We want to make this very consise otherwise the logs are not able to be read by humans.
    let format = tracing_subscriber::fmt::format::debug_fn(|writer, field, value| {
        if format!("{}", field) == "message" {
            write!(writer, "{}: {:?}", field, value)
        } else {
            write!(writer, "{}", field)
        }
    })
    // Separate each field with a comma.
    // This method is provided by an extension trait in the
    // `tracing-subscriber` prelude.
    .delimited(", ");

    let (json, plain) = if opts.json {
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

    let otlp_host = match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        Ok(val) => val,
        Err(_) => "http://localhost:4317".to_string(),
    };

    // otel uses async runtime, so it must be started after the runtime
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_endpoint(otlp_host))
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_resource(Resource::new(vec![KeyValue::new("service.name", "machine-api")])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

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

    if let Err(err) = run_cmd(&opts).await {
        bail!("running cmd `{:?}` failed: {:?}", &opts.subcmd, err);
    }

    Ok(())
}

async fn run_cmd(opts: &Opts) -> Result<()> {
    match &opts.subcmd {
        SubCommand::Server(s) => {
            crate::server::server(s, opts).await?;
        }
        SubCommand::ListUsbDevices => {
            let ports = serialport::available_ports().expect("No ports found!");
            for p in ports {
                println!("{}: {:?}", p.port_name, p.port_type);
            }
        }
        SubCommand::SliceFile { config_file, file } => {
            let gcode = GcodeSequence::from_stl_path(config_file, file)?;
            println!("Parsed {} lines of gcode", gcode.lines.len());
        }
        SubCommand::PrintFile { config_file, file } => {
            let extension = file.extension().unwrap_or(OsStr::new("stl"));
            let gcode = if extension != "gcode" {
                GcodeSequence::from_stl_path(config_file, file)?
            } else {
                GcodeSequence::from_file_path(file)?
            };

            // Now connect to printer over serial port
            let mut printer = UsbPrinter::new();
            printer.wait_for_start()?;

            for line in gcode.lines.iter() {
                let msg = format!("{}\r\n", line);
                println!("writing: {}", line);
                printer.writer.write_all(msg.as_bytes())?;
                printer.wait_for_ok()?;
            }
        }
    }

    Ok(())
}
