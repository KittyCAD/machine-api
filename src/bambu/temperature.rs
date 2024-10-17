use std::{collections::HashMap, sync::Arc};

use anyhow::Result;

use super::Bambu;
use crate::{TemperatureSensor, TemperatureSensorReading, TemperatureSensors as TemperatureSensorsTrait};

impl Bambu {
    /// Return a handle to read the temperature information from the
    /// Moonraker printer.
    pub fn get_temperature_sensors(&self) -> TemperatureSensors {
        TemperatureSensors {
            client: self.client.clone(),
        }
    }
}

/// Struct to read Temperature values from the 3d printer.
#[derive(Clone)]
pub struct TemperatureSensors {
    client: Arc<bambulabs::client::Client>,
}

impl TemperatureSensorsTrait for TemperatureSensors {
    type Error = anyhow::Error;

    async fn sensors(&self) -> Result<HashMap<String, TemperatureSensor>> {
        Ok(HashMap::from([
            ("extruder".to_owned(), TemperatureSensor::Extruder),
            ("bed".to_owned(), TemperatureSensor::Bed),
            ("chamber".to_owned(), TemperatureSensor::Chamber),
        ]))
    }

    async fn poll_sensors(&mut self) -> Result<HashMap<String, TemperatureSensorReading>> {
        let Some(status) = self.client.get_status()? else {
            return Ok(HashMap::new());
        };

        let mut sensor_readings = HashMap::from([(
            "extruder".to_owned(),
            TemperatureSensorReading {
                temperature_celsius: status.nozzle_temper.unwrap_or(0.0),
                target_temperature_celsius: status.nozzle_target_temper,
            },
        )]);

        sensor_readings.insert(
            "bed".to_owned(),
            TemperatureSensorReading {
                temperature_celsius: status.bed_temper.unwrap_or(0.0),
                target_temperature_celsius: status.bed_target_temper,
            },
        );

        sensor_readings.insert(
            "chamber".to_owned(),
            TemperatureSensorReading {
                temperature_celsius: status.chamber_temper.unwrap_or(0.0),
                target_temperature_celsius: None,
            },
        );

        Ok(sensor_readings)
    }
}
