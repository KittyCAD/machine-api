use anyhow::Result;
use machine_api::server;
use std::str::FromStr;
use tokio::sync::RwLock;
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

mod config;
use config::Config;

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
    let cfg: Config = serde_yaml::from_reader(std::fs::File::open(&args[1])?)?;

    // let cfg: PathBuf = args[3].parse().unwrap();

    // machines.insert(
    //     args[1].clone(),
    //     RwLock::new(Machine::new(
    //         moonraker::Client::neptune4(&args[2])?,
    //         prusa::Slicer::new(&cfg),
    //     )),
    // );

    server::serve(
        "0.0.0.0:8080",
        cfg.load()
            .await?
            .into_iter()
            .map(|(machine_id, machine)| (machine_id, RwLock::new(machine)))
            .collect(),
    )
    .await?;

    Ok(())
}
