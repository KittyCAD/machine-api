use super::{Cli, Config};
use anyhow::Result;
use machine_api::{moonraker, server, AnyMachine};
use prometheus_client::{
    metrics::gauge::Gauge,
    registry::{Registry, Unit},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{atomic::AtomicU64, Arc},
};
use tokio::sync::RwLock;

/// Long-term this should get a new trait, and a MachineT: Metrics / generic
/// param on this function.
///
/// For now we can just do this for moonraker (and maybe one or two others)
/// before we refine the API.
async fn spawn_metrics_moonraker(registry: &mut Registry, key: &str, machine: &moonraker::Client) {
    let registry = registry.sub_registry_with_label(("id".into(), key.to_owned().into()));

    let machine = machine.clone();

    let extruder_temperature = Gauge::<f64, AtomicU64>::default();
    registry.register_with_unit(
        "extruder_temperature",
        "Last temp of the extruder",
        Unit::Celsius,
        extruder_temperature.clone(),
    );

    let extruder_temperature_target = Gauge::<f64, AtomicU64>::default();
    registry.register_with_unit(
        "extruder_temperature_target",
        "Target temp of the extruder",
        Unit::Celsius,
        extruder_temperature_target.clone(),
    );

    let bed_temperature = Gauge::<f64, AtomicU64>::default();
    registry.register_with_unit(
        "bed_temperature",
        "Last temp of the bed",
        Unit::Celsius,
        bed_temperature.clone(),
    );

    let bed_temperature_target = Gauge::<f64, AtomicU64>::default();
    registry.register_with_unit(
        "bed_temperature_target",
        "Target temp of the bed",
        Unit::Celsius,
        bed_temperature_target.clone(),
    );

    let key = key.to_owned();
    tokio::spawn(async move {
        let key = key;
        let machine = machine;
        let extruder_temperature = extruder_temperature;
        let extruder_temperature_target = extruder_temperature_target;
        let bed_temperature = bed_temperature;
        let bed_temperature_target = bed_temperature_target;

        loop {
            let Ok(readings) = machine.get_client().temperatures().await else {
                tracing::warn!("failed to collect temperatures from {}", key);

                /* This mega-sucks. I really really *REALLY* hate this. I
                 * can't possibly explain just how much this pisses me off.
                 *
                 * We can't dynamically remove the key from the prob export(s)
                 * (which would be my preference here tbh, missing values is
                 * handled fine), and keeping the last value is a lie (yes
                 * its absolutely still pumping out 500c, doesn't matter the
                 * box is offline) -- but 0 is a REALLY bad value since it's
                 * a valid number we can (and should!) return, so translating 0
                 * into NULL isn't going to work either.
                 *
                 * I have no idea what the real fix is, but this ain't it. This
                 * just stops graphs from lying when the box goes offline. */

                extruder_temperature.set(0.0);
                extruder_temperature_target.set(0.0);
                bed_temperature.set(0.0);
                bed_temperature_target.set(0.0);

                continue;
            };
            tracing::trace!("metrics collected from {}", key);

            // TODO: collect last N values and avg?

            extruder_temperature.set(*readings.extruder.temperatures.last().unwrap_or(&0.0));
            extruder_temperature_target.set(*readings.extruder.targets.last().unwrap_or(&0.0));

            if let Some(heater_bed) = readings.heater_bed {
                bed_temperature.set(*heater_bed.temperatures.last().unwrap_or(&0.0));
                bed_temperature_target.set(*heater_bed.targets.last().unwrap_or(&0.0));
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
}

pub async fn main(_cli: &Cli, cfg: &Config, bind: &str) -> Result<()> {
    let machines = Arc::new(RwLock::new(HashMap::new()));

    cfg.spawn_discover_usb(machines.clone()).await?;
    cfg.spawn_discover_bambu(machines.clone()).await?;
    cfg.create_noop(machines.clone()).await?;
    cfg.create_moonraker(machines.clone()).await?;

    let mut registry = Registry::default();
    for (key, machine) in machines.read().await.iter() {
        let machine = machine.read().await;
        let any_machine = machine.get_machine();

        match &any_machine {
            AnyMachine::Moonraker(moonraker) => {
                spawn_metrics_moonraker(&mut registry, key, moonraker).await;
            }
            _ => { /* Nothing to do here! */ }
        }
    }

    let bind_addr: SocketAddr = bind.parse()?;
    tokio::spawn(async move {
        let bind_addr = bind_addr;
        let responder = libmdns::Responder::new().unwrap();
        let _svc = responder.register(
            "_machine-api._tcp".to_owned(),
            "Machine Api Server".to_owned(),
            bind_addr.port(),
            &["path=/"],
        );

        tracing::info!(
            bind_addr = bind_addr.to_string(),
            "starting mDNS advertisement for _machine-api._tcp"
        );
    });

    server::serve(bind, machines, registry).await?;
    Ok(())
}
