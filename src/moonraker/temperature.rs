use super::Client;
use crate::{TemperatureSensor, TemperatureSensorReading, TemperatureSensors as TemperatureSensorsTrait};
use anyhow::Result;
use std::collections::HashMap;

impl Client {
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
    client: moonraker::Client,
}

impl TemperatureSensorsTrait for TemperatureSensors {
    type Error = anyhow::Error;

    async fn sensors(&self) -> Result<HashMap<String, TemperatureSensor>> {
        Ok(HashMap::from([
            ("extruder".to_owned(), TemperatureSensor::Extruder),
            ("bed".to_owned(), TemperatureSensor::Bed),
        ]))
    }

    async fn poll_sensors(&mut self) -> Result<HashMap<String, TemperatureSensorReading>> {
        let readings = self.client.temperatures().await?;

        let mut sensor_readings = HashMap::from([(
            "extruder".to_owned(),
            TemperatureSensorReading {
                temperature_celsius: *readings.extruder.temperatures.last().unwrap_or(&0.0),
                target_temperature_celsius: Some(*readings.extruder.targets.last().unwrap_or(&0.0)),
            },
        )]);

        if let Some(heater_bed) = readings.heater_bed {
            sensor_readings.insert(
                "bed".to_owned(),
                TemperatureSensorReading {
                    temperature_celsius: *heater_bed.temperatures.last().unwrap_or(&0.0),
                    target_temperature_celsius: Some(*heater_bed.targets.last().unwrap_or(&0.0)),
                },
            );
        }

        Ok(sensor_readings)
    }
}
