use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{atomic::AtomicU64, Arc},
};

use anyhow::Result;
use machine_api::{server, AnyMachine, TemperatureSensors};
use prometheus_client::{
    metrics::gauge::Gauge,
    registry::{Registry, Unit},
};
use tokio::sync::RwLock;

use super::{Cli, Config};

/// Long-term this should get a new trait, and a MachineT: Metrics / generic
/// param on this function.
///
/// For now we can just do this for moonraker (and maybe one or two others)
/// before we refine the API.
async fn spawn_metrics<TemperatureSensorT>(
    registry: Arc<RwLock<Registry>>,
    key: &str,
    machine: TemperatureSensorT,
) -> Result<(), TemperatureSensorT::Error>
where
    TemperatureSensorT: TemperatureSensors,
    TemperatureSensorT: Send,
    TemperatureSensorT: 'static,
    TemperatureSensorT::Error: Send,
    TemperatureSensorT::Error: 'static,
{
    let mut registry = registry.write().await;

    let registry = registry.sub_registry_with_label(("id".into(), key.to_owned().into()));

    let mut sensors = HashMap::new();

    for (sensor_id, sensor_type) in machine.sensors().await? {
        let sensor_id_target = format!("{}_target", sensor_id);

        sensors.insert(sensor_id.to_owned(), Gauge::<f64, AtomicU64>::default());
        sensors.insert(sensor_id_target.clone(), Gauge::<f64, AtomicU64>::default());

        registry.register_with_unit(
            &sensor_id,
            format!("machine-api sensor {} for {}'s {:?}", sensor_id, key, sensor_type),
            Unit::Celsius,
            sensors.get(&sensor_id).unwrap().clone(),
        );

        registry.register_with_unit(
            &sensor_id_target,
            format!(
                "machine-api sensor target {} for {}'s {:?}",
                sensor_id, key, sensor_type
            ),
            Unit::Celsius,
            sensors.get(&sensor_id_target).unwrap().clone(),
        );
    }

    let key = key.to_owned();
    tokio::spawn(async move {
        let key = key;
        let mut machine = machine;
        let mut sensors = sensors;

        loop {
            let Ok(readings) = machine.poll_sensors().await else {
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

                for (_, gauge) in sensors.iter_mut() {
                    gauge.set(0.0);
                }

                continue;
            };
            tracing::trace!("metrics collected from {}", key);

            for (sensor_id, sensor_reading) in readings.iter() {
                let sensor_id_target = format!("{}_target", sensor_id);
                if let Some(gauge) = sensors.get(sensor_id) {
                    gauge.set(sensor_reading.temperature_celsius);
                }
                if let Some(gauge) = sensors.get(&sensor_id_target) {
                    if let Some(target_temperature_celsius) = sensor_reading.target_temperature_celsius {
                        gauge.set(target_temperature_celsius);
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    Ok(())
}

pub async fn main(_cli: &Cli, cfg: &Config, bind: &str) -> Result<()> {
    let machines = Arc::new(RwLock::new(HashMap::new()));

    let (found_send, found_recv) = tokio::sync::mpsc::channel::<String>(1);

    cfg.spawn_discover_usb(found_send.clone(), machines.clone()).await?;
    cfg.spawn_discover_bambu(found_send.clone(), machines.clone()).await?;
    cfg.create_noop(found_send.clone(), machines.clone()).await?;
    cfg.create_moonraker(found_send.clone(), machines.clone()).await?;

    let registry = Arc::new(RwLock::new(Registry::default()));

    let registry1 = registry.clone();
    let machines1 = machines.clone();
    tokio::spawn(async move {
        let machines = machines1;
        let mut found_recv = found_recv;
        let registry = registry1;

        while let Some(machine_id) = found_recv.recv().await {
            let machines_read = machines.read().await;
            let Some(machine) = machines_read.get(&machine_id) else {
                tracing::warn!("someone lied about {}", machine_id);
                continue;
            };

            let machine = machine.read().await;
            let any_machine = machine.get_machine();

            match &any_machine {
                AnyMachine::Moonraker(moonraker) => {
                    let _ = spawn_metrics(registry.clone(), &machine_id, moonraker.get_temperature_sensors()).await;
                }
                _ => { /* Nothing to do here! */ }
            }
        }
    });

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
