//! An API server for controlling various machines.

#![deny(missing_docs)]

mod config;
mod machine;
mod network_printer;
mod print_manager;
mod server;
mod slicer;
#[cfg(test)]
mod tests;
mod usb_printer;

use std::sync::Arc;

use anyhow::{bail, Result};
use clap::Parser;
use config::Config;
use machine::MachineHandle;
use opentelemetry::{trace::TracerProvider, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use server::context::{discovery, Context};
use slog::Drain;
use tracing_subscriber::prelude::*;

const TIMEOUT_DURATION: std::time::Duration = std::time::Duration::from_secs(10);

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

    /// Path to config file.
    #[clap(short, long, default_value = "machine-api.toml")]
    pub config: std::path::PathBuf,
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

    /// List all available machines on the network or over USB.
    ListMachines,

    /// Print the given `file` with config from `config_file`
    PrintFile {
        /// Id for a machine
        machine_id: String,

        /// File path to slice
        file: std::path::PathBuf,

        /// The name of the job.
        /// If not given, the file name will be used.
        #[clap(long)]
        job_name: Option<String>,
    },

    /// Get machine status.
    GetStatus {
        /// Id for a machine
        machine_id: String,
    },

    /// Get machine version.
    GetVersion {
        /// Id for a machine
        machine_id: String,
    },

    /// Get machine accessories.
    GetAccessories {
        /// Id for a machine
        machine_id: String,
    },

    /// Turn off/on the machine led.
    LedControl {
        /// Id for a machine
        machine_id: String,

        /// Turn on or off
        #[clap(long)]
        on: bool,
    },

    /// Pause the machine.
    Pause {
        /// Id for a machine
        machine_id: String,
    },

    /// Resume the machine.
    Resume {
        /// Id for a machine
        machine_id: String,
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
        (Some(tracing_subscriber::fmt::layer().json()), None)
    } else {
        (None, Some(tracing_subscriber::fmt::layer().pretty().fmt_fields(format)))
    };

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

    // Initialize tracing.
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
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

    let config = config::Config::from_file(&opts.config)?;

    if let Err(err) = run_cmd(&opts, &config).await {
        bail!("running cmd `{:?}` failed: {:?}", &opts.subcmd, err);
    }

    Ok(())
}

async fn run_cmd(opts: &Opts, config: &Config) -> Result<()> {
    match &opts.subcmd {
        SubCommand::Server(s) => {
            crate::server::server(s, opts, config).await?;
        }
        SubCommand::ListMachines => {
            // Now connect to first printer we find over serial port
            //
            let api_context = Arc::new(Context::new(config, Default::default(), opts.create_logger("print")).await?);

            discovery(api_context.clone(), TIMEOUT_DURATION).await?;

            let machines = api_context.list_machines()?;
            for (id, machine) in machines.iter() {
                println!("{}: {:?}", id, machine);
            }
        }
        SubCommand::PrintFile {
            machine_id,
            file,
            job_name,
        } => {
            // Now connect to first printer we find over serial port
            let api_context = Arc::new(Context::new(config, Default::default(), opts.create_logger("print")).await?);

            discovery(api_context.clone(), TIMEOUT_DURATION).await?;

            let machine = api_context
                .find_machine_handle_by_id(machine_id)?
                .expect("Printer not found by given ID");

            let job_name = job_name.as_deref().unwrap_or_else(|| {
                file.file_name()
                    .ok_or_else(|| anyhow::anyhow!("file name not found"))
                    .unwrap()
                    .to_str()
                    .unwrap()
            });

            let result = machine.slice_and_print(job_name, file).await?;
            println!("{:#?}", result);
        }
        SubCommand::GetStatus { machine_id } => {
            // Now connect to first printer we find over serial port
            //
            let api_context = Arc::new(Context::new(config, Default::default(), opts.create_logger("print")).await?);

            discovery(api_context.clone(), TIMEOUT_DURATION).await?;

            let machine = api_context
                .find_machine_handle_by_id(machine_id)?
                .expect("Printer not found by given ID");

            let MachineHandle::NetworkPrinter(machine) = machine else {
                bail!("usb printers not yet supported");
            };

            let status = machine.client.status().await?;

            println!("{:#?}", status);
        }
        SubCommand::GetVersion { machine_id } => {
            // Now connect to first printer we find over serial port
            //
            let api_context = Arc::new(Context::new(config, Default::default(), opts.create_logger("print")).await?);

            discovery(api_context.clone(), TIMEOUT_DURATION).await?;

            let machine = api_context
                .find_machine_handle_by_id(machine_id)?
                .expect("Printer not found by given ID");

            let MachineHandle::NetworkPrinter(machine) = machine else {
                bail!("usb printers not yet supported");
            };

            let status = machine.client.version().await?;

            println!("{:#?}", status);
        }
        SubCommand::LedControl { machine_id, on } => {
            // Now connect to first printer we find over serial port
            //
            let api_context = Arc::new(Context::new(config, Default::default(), opts.create_logger("print")).await?);

            discovery(api_context.clone(), TIMEOUT_DURATION).await?;

            let machine = api_context
                .find_machine_handle_by_id(machine_id)?
                .expect("Printer not found by given ID");

            let MachineHandle::NetworkPrinter(machine) = machine else {
                bail!("usb printers not yet supported");
            };

            let status = machine.client.set_led(*on).await?;

            println!("{:#?}", status);
        }
        SubCommand::GetAccessories { machine_id } => {
            // Now connect to first printer we find over serial port
            //
            let api_context = Arc::new(Context::new(config, Default::default(), opts.create_logger("print")).await?);

            discovery(api_context.clone(), TIMEOUT_DURATION).await?;

            let machine = api_context
                .find_machine_handle_by_id(machine_id)?
                .expect("Printer not found by given ID");

            let MachineHandle::NetworkPrinter(machine) = machine else {
                bail!("usb printers not yet supported");
            };

            let status = machine.client.accessories().await?;

            println!("{:#?}", status);
        }
        SubCommand::Pause { machine_id } => {
            // Now connect to first printer we find over serial port
            //
            let api_context = Arc::new(Context::new(config, Default::default(), opts.create_logger("print")).await?);

            discovery(api_context.clone(), TIMEOUT_DURATION).await?;

            let machine = api_context
                .find_machine_handle_by_id(machine_id)?
                .expect("Printer not found by given ID");

            let MachineHandle::NetworkPrinter(machine) = machine else {
                bail!("usb printers not yet supported");
            };

            let status = machine.client.pause().await?;

            println!("{:#?}", status);
        }
        SubCommand::Resume { machine_id } => {
            // Now connect to first printer we find over serial port
            //
            let api_context = Arc::new(Context::new(config, Default::default(), opts.create_logger("print")).await?);

            discovery(api_context.clone(), TIMEOUT_DURATION).await?;

            let machine = api_context
                .find_machine_handle_by_id(machine_id)?
                .expect("Printer not found by given ID");

            let MachineHandle::NetworkPrinter(machine) = machine else {
                bail!("usb printers not yet supported");
            };

            let status = machine.client.resume().await?;

            println!("{:#?}", status);
        }
    }

    Ok(())
}
