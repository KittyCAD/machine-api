//! Code for the configuration of the application.

use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// The configuration of the application.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// The configuration for bambu labs machines.
    pub bambulabs: Option<BambuLabsConfig>,
    /// The configuration for formlabs machines.
    pub formlabs: Option<FormLabsConfig>,
}

impl Config {
    /// Parse a configuration from a toml file.
    pub fn from_file(file: &PathBuf) -> Result<Self> {
        let config = std::fs::read_to_string(file)?;
        Self::from_str(&config)
    }

    /// Parse a configuration from a toml string.
    pub fn from_str(config: &str) -> Result<Self> {
        Ok(toml::from_str(config)?)
    }
}

/// The configuration for bambu labs machines.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BambuLabsConfig {
    /// The machine ids and access codes for communication of LAN.
    pub machines: Vec<BambuLabsMachineConfig>,
}

impl BambuLabsConfig {
    /// Get the access code for a machine.
    pub fn get_access_code(&self, id: &str) -> Option<String> {
        self.machines.iter().find(|m| m.id == id).map(|m| m.access_code.clone())
    }
}

/// The configuration for a single bambu labs machine.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BambuLabsMachineConfig {
    /// The machine id.
    pub id: String,
    /// The access code for the machine.
    pub access_code: String,
}

/// The configuration for formlabs machines.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormLabsConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_str_no_formlabs() {
        let config = r#"
            [bambulabs]
            machines = [
                { id = "1", access_code = "1234" },
                { id = "2", access_code = "5678" },
            ]
        "#;
        let config = Config::from_str(config).unwrap();
        assert!(config.bambulabs.is_some());
        let bl = config.bambulabs.unwrap();
        assert_eq!(bl.machines.len(), 2);
        assert_eq!(bl.get_access_code("1").unwrap(), "1234");
        assert_eq!(bl.get_access_code("2").unwrap(), "5678");
        assert_eq!(bl.get_access_code("3"), None);

        assert!(config.formlabs.is_none());
    }

    #[test]
    fn test_config_from_str_with_formlabs() {
        let config = r#"[bambulabs]
machines = [
    { id = "1", access_code = "1234" },
    { id = "2", access_code = "5678" },
]

[formlabs] 
        "#;
        let config = Config::from_str(config).unwrap();
        assert!(config.bambulabs.is_some());
        let bl = config.bambulabs.unwrap();
        assert_eq!(bl.machines.len(), 2);
        assert_eq!(bl.get_access_code("1").unwrap(), "1234");
        assert_eq!(bl.get_access_code("2").unwrap(), "5678");
        assert_eq!(bl.get_access_code("3"), None);
        assert!(config.formlabs.is_some());
    }
}
