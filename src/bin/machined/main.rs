use anyhow::Result;
use machine_api::{moonraker, server, slicer::prusa, Machine};
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use tokio::sync::RwLock;
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

mod config;

// run like: machined foo http://foo.local cfg/prusa/foo.ini
#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::from_str("info").unwrap())
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args: Vec<String> = std::env::args().collect();
    let cfg: PathBuf = args[3].parse().unwrap();

    let mut machines = HashMap::<String, RwLock<Machine>>::new();
    machines.insert(
        args[1].clone(),
        RwLock::new(Machine::new(
            moonraker::Client::neptune4(&args[2])?,
            prusa::Slicer::new(&cfg),
        )),
    );

    server::serve("0.0.0.0:8080", machines).await?;

    Ok(())
}
