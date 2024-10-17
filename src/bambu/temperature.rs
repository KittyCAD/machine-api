use super::X1Carbon;
use crate::{TemperatureSensor, TemperatureSensorReading, TemperatureSensors as TemperatureSensorsTrait};
use anyhow::Result;
use std::collections::HashMap;

impl TemperatureSensorsTrait for X1Carbon {
    type Error = anyhow::Error;

    async fn sensors(&self) -> Result<HashMap<String, TemperatureSensor>> {
        Ok(HashMap::from([
            ("extruder".to_owned(), TemperatureSensor::Extruder),
            ("bed".to_owned(), TemperatureSensor::Bed),
            ("chamber".to_owned(), TemperatureSensor::Chamber),
        ]))
    }

    async fn poll_sensors(&mut self) -> Result<HashMap<String, TemperatureSensorReading>> {
        let Some(status) = self.get_status()? else {
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
