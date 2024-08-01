//! The commands that can be sent to the printer.

use parse_display::{Display, FromStr};
use serde::{Deserialize, Serialize};

use crate::{sequence_id::SequenceId, speedprofile::SpeedProfile};

/// The commands that can be sent to the printer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    /// An information command.
    Info(Info),
    /// A print command.
    Print(Print),
    /// A system command.
    System(System),
    /// A pushing command.
    Pushing(Pushing),
}

impl Command {
    /// Get the sequence ID.
    pub fn sequence_id(&self) -> &SequenceId {
        match self {
            Command::Info(info) => info.sequence_id(),
            Command::Print(print) => print.sequence_id(),
            Command::System(system) => system.sequence_id(),
            Command::Pushing(pushing) => pushing.sequence_id(),
        }
    }

    /// Return a command to get the version of the printer.
    pub fn get_version() -> Self {
        Command::Info(Info::GetVersion(GetVersion {
            sequence_id: SequenceId::new(),
        }))
    }

    /// Return a command to push all data.
    pub fn push_all() -> Self {
        Command::Pushing(Pushing::Pushall(Pushall {
            sequence_id: SequenceId::new(),
        }))
    }

    /// Return a command to pause the current print.
    pub fn pause() -> Self {
        Command::Print(Print::Pause(Pause {
            sequence_id: SequenceId::new(),
        }))
    }

    /// Return a command to resume the current print.
    pub fn resume() -> Self {
        Command::Print(Print::Resume(Resume {
            sequence_id: SequenceId::new(),
        }))
    }

    /// Return a command to stop the current print.
    pub fn stop() -> Self {
        Command::Print(Print::Stop(Stop {
            sequence_id: SequenceId::new(),
        }))
    }

    /// Return a command to set the speed profile.
    pub fn set_speed_profile(profile: SpeedProfile) -> Self {
        Command::Print(Print::PrintSpeed(PrintSpeed {
            sequence_id: SequenceId::new(),
            param: profile,
        }))
    }

    /// Return a command to send a GCode line.
    pub fn send_gcode_line(line: &str) -> Self {
        Command::Print(Print::GcodeLine(GcodeLine {
            sequence_id: SequenceId::new(),
            param: line.to_string(),
        }))
    }

    /// Return a command to set the chamber light.
    pub fn set_chamber_light(led_mode: LedMode) -> Self {
        Command::System(System::Ledctrl(Ledctrl {
            sequence_id: SequenceId::new(),
            led_node: LedNode::ChamberLight,
            led_mode,
            led_on_time: 500,
            led_off_time: 500,
            loop_times: 0,
            interval_time: 0,
        }))
    }

    /// Return a command to get accessories.
    pub fn get_accessories() -> Self {
        Command::System(System::GetAccessories(GetAccessories {
            sequence_id: SequenceId::new(),
            accessory_type: AccessoryType::None,
        }))
    }
}

/// An information command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "command")]
pub enum Info {
    /// Get the version of the printer.
    GetVersion(GetVersion),
}

impl Info {
    /// Get the sequence ID.
    pub fn sequence_id(&self) -> &SequenceId {
        match self {
            Info::GetVersion(GetVersion { sequence_id }) => sequence_id,
        }
    }
}

/// A print command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "command")]
pub enum Print {
    /// Pause the current print.
    Pause(Pause),
    /// Resume the current print.
    Resume(Resume),
    /// Stop the current print.
    Stop(Stop),
    /// Set the speed profile.
    PrintSpeed(PrintSpeed),
    /// Send a GCode file.
    GcodeLine(GcodeLine),
}

impl Print {
    /// Get the sequence ID.
    pub fn sequence_id(&self) -> &SequenceId {
        match self {
            Print::Pause(Pause { sequence_id }) => sequence_id,
            Print::Resume(Resume { sequence_id }) => sequence_id,
            Print::Stop(Stop { sequence_id }) => sequence_id,
            Print::PrintSpeed(PrintSpeed { sequence_id, .. }) => sequence_id,
            Print::GcodeLine(GcodeLine { sequence_id, .. }) => sequence_id,
        }
    }
}

/// A system command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "command")]
pub enum System {
    /// Set the chamber light.
    Ledctrl(Ledctrl),
    /// Get accessories.
    GetAccessories(GetAccessories),
}

impl System {
    /// Get the sequence ID.
    pub fn sequence_id(&self) -> &SequenceId {
        match self {
            System::Ledctrl(Ledctrl { sequence_id, .. }) => sequence_id,
            System::GetAccessories(GetAccessories { sequence_id, .. }) => sequence_id,
        }
    }
}

/// A pushing command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "command")]
pub enum Pushing {
    /// Get all device information.
    Pushall(Pushall),
    /// Start pushing data.
    Start(Start),
}

impl Pushing {
    /// Get the sequence ID.
    pub fn sequence_id(&self) -> &SequenceId {
        match self {
            Pushing::Pushall(Pushall { sequence_id }) => sequence_id,
            Pushing::Start(Start { sequence_id }) => sequence_id,
        }
    }
}

/// The payload for getting the version of the printer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetVersion {
    /// The sequence ID.
    pub sequence_id: SequenceId,
}

/// The payload for pausing the current print.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pause {
    /// The sequence ID.
    pub sequence_id: SequenceId,
}

/// The payload for resuming the current print.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resume {
    /// The sequence ID.
    pub sequence_id: SequenceId,
}

/// The payload for stopping the current print.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stop {
    /// The sequence ID.
    pub sequence_id: SequenceId,
}

/// The payload for getting all device information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pushall {
    /// The sequence ID.
    pub sequence_id: SequenceId,
}

/// The payload for starting to push data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Start {
    /// The sequence ID.
    pub sequence_id: SequenceId,
}

/// The payload for setting led control.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ledctrl {
    /// The sequence ID.
    pub sequence_id: SequenceId,
    /// The LED node.
    pub led_node: LedNode,
    /// The LED mode.
    pub led_mode: LedMode,
    /// The LED on time.
    pub led_on_time: u32,
    /// The LED off time.
    pub led_off_time: u32,
    /// The loop times.
    pub loop_times: u32,
    /// The interval time.
    pub interval_time: u32,
}

/// The node for the led.
#[derive(Debug, Clone, Serialize, Deserialize, Display, FromStr, PartialEq, Eq)]
#[display(style = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LedNode {
    /// The chamber light.
    ChamberLight,
}

/// The mode for the led.
#[derive(Debug, Clone, Serialize, Deserialize, Display, FromStr, PartialEq, Eq)]
#[display(style = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LedMode {
    /// Turn the LED on.
    On,
    /// Turn the LED off.
    Off,
}

impl From<bool> for LedMode {
    fn from(on: bool) -> Self {
        if on {
            LedMode::On
        } else {
            LedMode::Off
        }
    }
}

/// The payload for setting the speed profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrintSpeed {
    /// The sequence ID.
    pub sequence_id: SequenceId,
    /// The profile.
    pub param: SpeedProfile,
}

/// The payload for sending a GCode file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GcodeLine {
    /// The sequence ID.
    pub sequence_id: SequenceId,
    /// The GCode.
    pub param: String,
}

/// The payload for getting accessories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetAccessories {
    /// The sequence ID.
    pub sequence_id: SequenceId,
    /// The accessory type.
    pub accessory_type: AccessoryType,
}

/// The type of accessory.
#[derive(Debug, Clone, Serialize, Deserialize, Display, FromStr, PartialEq, Eq)]
#[display(style = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AccessoryType {
    /// No accessory.
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_get_version() {
        let uid = SequenceId::new();
        let payload = format!(r#"{{"info": {{"sequence_id": {uid}, "command": "get_version"}}}}"#);
        let command: Command = serde_json::from_str(&payload).unwrap();
        if let Command::Info(Info::GetVersion(GetVersion { sequence_id })) = command {
            assert_eq!(sequence_id, uid);
        } else {
            panic!("Invalid command deserialized");
        }
    }

    #[test]
    fn test_deserialize_pause() {
        let uid = SequenceId::new();
        let payload = format!(r#"{{"print": {{"sequence_id": {uid}, "command": "pause"}}}}"#);
        let command: Command = serde_json::from_str(&payload).unwrap();
        if let Command::Print(Print::Pause(Pause { sequence_id })) = command {
            assert_eq!(sequence_id, uid);
        } else {
            panic!("Invalid command deserialized");
        }
    }

    #[test]
    fn test_deserialize_resume() {
        let uid = SequenceId::new();
        let payload = format!(r#"{{"print": {{"sequence_id": {uid}, "command": "resume"}}}}"#);
        let command: Command = serde_json::from_str(&payload).unwrap();
        if let Command::Print(Print::Resume(Resume { sequence_id })) = command {
            assert_eq!(sequence_id, uid);
        } else {
            panic!("Invalid command deserialized");
        }
    }

    #[test]
    fn test_deserialize_stop() {
        let uid = SequenceId::new();
        let payload = format!(r#"{{"print": {{"sequence_id": {uid}, "command": "stop"}}}}"#);
        let command: Command = serde_json::from_str(&payload).unwrap();
        if let Command::Print(Print::Stop(Stop { sequence_id })) = command {
            assert_eq!(sequence_id, uid);
        } else {
            panic!("Invalid command deserialized");
        }
    }

    #[test]
    fn test_deserialize_pushall() {
        let uid = SequenceId::new();
        let payload = format!(r#"{{"pushing": {{"sequence_id": {uid}, "command": "pushall"}}}}"#);
        let command: Command = serde_json::from_str(&payload).unwrap();
        if let Command::Pushing(Pushing::Pushall(Pushall { sequence_id })) = command {
            assert_eq!(sequence_id, uid);
        } else {
            panic!("Invalid command deserialized");
        }
    }

    #[test]
    fn test_deserialize_start() {
        let uid = SequenceId::new();
        let payload = format!(r#"{{"pushing": {{"sequence_id": {uid}, "command": "start"}}}}"#);
        let command: Command = serde_json::from_str(&payload).unwrap();
        if let Command::Pushing(Pushing::Start(Start { sequence_id })) = command {
            assert_eq!(sequence_id, uid);
        } else {
            panic!("Invalid command deserialized");
        }
    }

    #[test]
    fn test_deserialize_ledctrl() {
        let uid = SequenceId::new();
        let payload = format!(
            r#"{{"system": {{"sequence_id": {uid}, "command": "ledctrl", "led_node": "chamber_light", "led_mode": "on", "led_on_time": 500, "led_off_time": 500, "loop_times": 0, "interval_time": 0}}}}"#,
            uid = uid
        );
        let command: Command = serde_json::from_str(&payload).unwrap();
        if let Command::System(System::Ledctrl(Ledctrl {
            sequence_id,
            led_node,
            led_mode,
            led_on_time,
            led_off_time,
            loop_times,
            interval_time,
        })) = command
        {
            assert_eq!(sequence_id, uid);
            assert_eq!(led_node, LedNode::ChamberLight);
            assert_eq!(led_mode, LedMode::On);
            assert_eq!(led_on_time, 500);
            assert_eq!(led_off_time, 500);
            assert_eq!(loop_times, 0);
            assert_eq!(interval_time, 0);
        } else {
            panic!("Invalid command deserialized");
        }
    }

    #[test]
    fn test_deserialize_print_speed() {
        let uid = SequenceId::new();
        let payload = format!(
            r#"{{"print": {{"sequence_id": {uid}, "command": "print_speed", "param": "standard"}}}}"#,
            uid = uid
        );
        let command: Command = serde_json::from_str(&payload).unwrap();
        if let Command::Print(Print::PrintSpeed(PrintSpeed { sequence_id, param })) = command {
            assert_eq!(sequence_id, uid);
            assert_eq!(param, SpeedProfile::Standard);
        } else {
            panic!("Invalid command deserialized");
        }
    }

    #[test]
    fn test_deserialize_gcode_line() {
        let uid = SequenceId::new();
        let payload = format!(
            r#"{{"print": {{"sequence_id": {uid}, "command": "gcode_line", "param": "G28"}}}}"#,
            uid = uid
        );
        let command: Command = serde_json::from_str(&payload).unwrap();
        if let Command::Print(Print::GcodeLine(GcodeLine { sequence_id, param })) = command {
            assert_eq!(sequence_id, uid);
            assert_eq!(param, "G28");
        } else {
            panic!("Invalid command deserialized");
        }
    }

    #[test]
    fn test_deserialize_get_accessories() {
        let uid = SequenceId::new();
        let payload = format!(
            r#"{{"system": {{"sequence_id": {uid}, "command": "get_accessories", "accessory_type": "none"}}}}"#,
            uid = uid
        );
        let command: Command = serde_json::from_str(&payload).unwrap();
        if let Command::System(System::GetAccessories(GetAccessories {
            sequence_id,
            accessory_type,
        })) = command
        {
            assert_eq!(sequence_id, uid);
            assert_eq!(accessory_type, AccessoryType::None);
        } else {
            panic!("Invalid command deserialized");
        }
    }

    #[test]
    fn test_serialize_get_version() {
        let uid = SequenceId::new();
        let command = Command::Info(Info::GetVersion(GetVersion {
            sequence_id: uid.clone(),
        }));
        let payload = serde_json::to_string(&command).unwrap();
        assert_eq!(
            payload,
            format!(
                r#"{{"info":{{"command":"get_version","sequence_id":{uid}}}}}"#,
                uid = uid
            )
        );
    }

    #[test]
    fn test_serialize_pause() {
        let uid = SequenceId::new();
        let command = Command::Print(Print::Pause(Pause {
            sequence_id: uid.clone(),
        }));
        let payload = serde_json::to_string(&command).unwrap();
        assert_eq!(
            payload,
            format!(r#"{{"print":{{"command":"pause","sequence_id":{uid}}}}}"#, uid = uid)
        );
    }
}
