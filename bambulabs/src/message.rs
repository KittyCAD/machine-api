//! A message from the printer.

use parse_display::{Display, FromStr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::sequence_id::SequenceId;

/// A message from/to the printer.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Message {
    /// A print message.
    Print(Print),
    /// An info message.
    Info(Info),
    /// A system message.
    System(System),
    /// An unknown Json message.
    Json(Value),
    /// The message could not be parsed. The `Option<String>` contains the raw message.
    /// If the message could not be parsed as a string, the `Option` will be `None`.
    Unknown(Option<String>),
}

impl Message {
    /// Returns the sequence id of the message.
    pub fn sequence_id(&self) -> Option<SequenceId> {
        match self {
            Message::Print(print) => Some(print.sequence_id.clone()),
            Message::Info(info) => Some(info.sequence_id.clone()),
            Message::System(system) => Some(system.sequence_id.clone()),
            Message::Json(_) | Message::Unknown(_) => None,
        }
    }
}

/// A print message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Print {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The command.
    pub command: PrintCommand,
    /// The upload.
    pub upload: Option<PrintUpload>,
    /// The nozzle temperature.
    pub nozzle_temper: Option<f64>,
    /// The target nozzle temperature.
    pub nozzle_target_temper: Option<f64>,
    /// The bed temperature.
    pub bed_temper: Option<f64>,
    /// The target bed temperature.
    pub bed_target_temper: Option<f64>,
    /// The chamber temperature.
    pub chamber_temper: Option<f64>,
    /// The print stage.
    pub mc_print_stage: Option<String>,
    /// The heatbreak fan speed.
    pub heatbreak_fan_speed: Option<String>,
    /// The cooling fan speed.
    pub cooling_fan_speed: Option<String>,
    /// The big fan 1 speed.
    pub big_fan1_speed: Option<String>,
    /// The big fan 2 speed.
    pub big_fan2_speed: Option<String>,
    /// The percentage of the print completed.
    pub mc_percent: Option<i64>,
    /// The remaining time of the print.
    pub mc_remaining_time: Option<i64>,
    /// The ams status.
    pub ams_status: Option<i64>,
    /// The ams rfid status.
    pub ams_rfid_status: Option<i64>,
    /// The hw switch state.
    pub hw_switch_state: Option<i64>,
    /// The spd mag.
    pub spd_mag: Option<i64>,
    /// The spd lvl.
    pub spd_lvl: Option<i64>,
    /// The print error.
    pub print_error: Option<i64>,
    /// The lifecycle.
    pub lifecycle: Option<String>,
    /// The wifi signal.
    pub wifi_signal: Option<String>,
    /// The gcode state.
    pub gcode_state: Option<String>,
    /// The gcode file prepare percent.
    pub gcode_file_prepare_percent: Option<String>,
    /// The queue number.
    pub queue_number: Option<i64>,
    /// The queue total.
    pub queue_total: Option<i64>,
    /// The queue est.
    pub queue_est: Option<i64>,
    /// The queue sts.
    pub queue_sts: Option<i64>,
    /// The project id.
    pub project_id: Option<String>,
    /// The profile id.
    pub profile_id: Option<String>,
    /// The task id.
    pub task_id: Option<String>,
    /// The subtask id.
    pub subtask_id: Option<String>,
    /// The subtask name.
    pub subtask_name: Option<String>,
    /// The gcode file.
    pub gcode_file: Option<String>,
    /// The stg.
    pub stg: Option<Vec<Value>>,
    /// The stg cur.
    pub stg_cur: Option<i64>,
    /// The print type.
    pub print_type: Option<String>,
    /// The home flag.
    pub home_flag: Option<i64>,
    /// The mc print line number.
    pub mc_print_line_number: Option<String>,
    /// The mc print sub stage.
    pub mc_print_sub_stage: Option<i64>,
    /// Sdcard?
    pub sdcard: Option<bool>,
    /// Force upgrade?
    pub force_upgrade: Option<bool>,
    /// The mess production state.
    pub mess_production_state: Option<String>,
    /// The layer num.
    pub layer_num: Option<i64>,
    /// The total layer num.
    pub total_layer_num: Option<i64>,
    /// The s obj.
    pub s_obj: Option<Vec<Value>>,
    /// The fan gear.
    pub fan_gear: Option<i64>,
    /// The hms.
    pub hms: Option<Vec<Value>>,
    /// Online status.
    pub online: Option<PrintOnline>,
    /// The ams.
    pub ams: Option<PrintAms>,
    /// The ipcam.
    pub ipcam: Option<PrintIpcam>,
    /// The tray.
    pub vt_tray: Option<PrintTray>,
    /// The lights report.
    pub lights_report: Option<Vec<PrintLightsReport>>,
    /// The upgrade state.
    pub upgrade_state: Option<PrintUpgradeState>,
    /// The message.
    pub msg: Option<i64>,
}

/// A print command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Display, FromStr)]
#[serde(rename_all = "snake_case")]
#[display(style = "snake_case")]
pub enum PrintCommand {
    /// The status of the print.
    PushStatus,
    /// The gcode line.
    GcodeLine,
}

/// The print upload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PrintUpload {
    /// The status.
    pub status: String,
    /// The progress.
    pub progress: i64,
    /// The message.
    pub message: String,
}

/// The print online.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PrintOnline {
    /// The ahb.
    pub ahb: bool,
    /// The rfid.
    pub rfid: Option<bool>,
    /// The version.
    pub version: i64,
}

/// The print ams.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PrintAms {
    /// The ams.
    pub ams: Option<Vec<PrintAmsData>>,
    /// The ams exist bits.
    pub ams_exist_bits: Option<String>,
    /// The tray exist bits.
    pub tray_exist_bits: Option<String>,
    /// The tray is bbl bits.
    pub tray_is_bbl_bits: Option<String>,
    /// The tray tar.
    pub tray_tar: Option<String>,
    /// The tray now.
    pub tray_now: Option<String>,
    /// The tray pre.
    pub tray_pre: Option<String>,
    /// The tray read done bits.
    pub tray_read_done_bits: Option<String>,
    /// The tray reading bits.
    pub tray_reading_bits: Option<String>,
    /// The version.
    pub version: Option<i64>,
    /// The insert flag.
    pub insert_flag: Option<bool>,
    /// The power on flag.
    pub power_on_flag: Option<bool>,
}

/// The print ams data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PrintAmsData {
    /// The id.
    pub id: String,
    /// The humidity.
    pub humidity: String,
    /// The temperature.
    pub temp: String,
    /// The tray.
    pub tray: Vec<PrintTray>,
}

/// The print tray.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PrintTray {
    /// The id.
    pub id: String,
    /// The tag uid.
    pub tag_uid: Option<String>,
    /// The tray id name.
    pub tray_id_name: Option<String>,
    /// The tray info index.
    pub tray_info_idx: Option<String>,
    /// The tray type.
    pub tray_type: Option<String>,
    /// The tray sub brands.
    pub tray_sub_brands: Option<String>,
    /// The tray color.
    pub tray_color: Option<String>,
    /// The tray weight.
    pub tray_weight: Option<String>,
    /// The tray diameter.
    pub tray_diameter: Option<String>,
    /// The tray temperature.
    pub tray_temp: Option<String>,
    /// The tray time.
    pub tray_time: Option<String>,
    /// The bed temperature type.
    pub bed_temp_type: Option<String>,
    /// The bed temperature.
    pub bed_temp: Option<String>,
    /// The nozzle temperature max.
    pub nozzle_temp_max: Option<String>,
    /// The nozzle temperature min.
    pub nozzle_temp_min: Option<String>,
    /// The xcam info.
    pub xcam_info: Option<String>,
    /// The tray uuid.
    pub tray_uuid: Option<String>,
    /// The tray remain.
    pub remain: Option<i64>,
    /// The tray k.
    pub k: Option<f64>,
    /// The tray n.
    pub n: Option<i64>,
}

/// The print ipcam.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PrintIpcam {
    /// The ipcam dev.
    pub ipcam_dev: Option<String>,
    /// The ipcam record.
    pub ipcam_record: Option<String>,
    /// The timelapse.
    pub timelapse: Option<String>,
    /// The mode bits.
    pub mode_bits: Option<i64>,
}

/// A print lights report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PrintLightsReport {
    /// The node.
    pub node: String,
    /// The mode.
    pub mode: String,
}

/// A print upgrade state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PrintUpgradeState {
    /// The sequence id.
    pub sequence_id: Option<i64>,
    /// The progress.
    pub progress: Option<String>,
    /// The status.
    pub status: Option<String>,
    /// The consistency request.
    pub consistency_request: Option<bool>,
    /// The dis state.
    pub dis_state: Option<i64>,
    /// The error code.
    pub err_code: Option<i64>,
    /// Force upgrade?
    pub force_upgrade: Option<bool>,
    /// The message.
    pub message: Option<String>,
    /// The module.
    pub module: Option<String>,
    /// The new version state.
    pub new_version_state: Option<i64>,
    /// The new version list.
    pub new_ver_list: Option<Vec<Value>>,
}

/// A info message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Info {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The info command.
    pub command: InfoCommand,
    /// The info module.
    pub module: Vec<InfoModule>,
    /// The result of the info command.
    pub result: String,
    /// The reason of the info command.
    pub reason: String,
}

/// An info command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Display, FromStr)]
#[serde(rename_all = "snake_case")]
#[display(style = "snake_case")]
pub enum InfoCommand {
    /// Get the version.
    GetVersion,
}

/// An info module.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InfoModule {
    /// The module name.
    pub name: String,
    /// The project name.
    pub project_name: String,
    /// The software version.
    pub sw_ver: String,
    /// The hardware version.
    pub hw_ver: String,
    /// The serial number.
    pub sn: String,
    /// The loader version.
    pub loader_ver: Option<String>,
    /// The ota version.
    pub ota_ver: Option<String>,
}

/// A system message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct System {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The system command.
    pub command: SystemCommand,
    /// The access code.
    pub access_code: Option<String>,
    /// The result of the system command.
    pub result: String,
}

/// A system command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, FromStr, Display)]
#[serde(rename_all = "snake_case")]
#[display(style = "snake_case")]
pub enum SystemCommand {
    /// Led control.
    Ledctrl,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_message_json() {
        let message = r#"{ "hello": "world" }"#;

        let result = serde_json::from_str::<Message>(message);

        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_message_print() {
        let message = format!(
            r#"{{ "print": {{ "bed_temper": 17.40625, "wifi_signal": "-59dBm", "command": "push_status", "msg": 1, "sequence_id": {} }}}}"#,
            2
        );

        let result = serde_json::from_str::<Message>(&message);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Message::Print(_)));
    }

    #[test]
    fn test_deserialize_message_info() {
        let message = format!(
            r#"{{
                "info":{{
                    "command":"get_version",
                    "sequence_id":{},
                    "module":[
                        {{
                            "name":"ota",
                            "project_name":"C11",
                            "sw_ver":"01.04.02.00",
                            "hw_ver":"OTA",
                            "sn":"01S00C123400001"
                        }}
                    ],
                    "result":"success",
                    "reason":""
                }}
            }}"#,
            2
        );

        let result = serde_json::from_str::<Message>(&message);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Message::Info(_)));
    }

    #[test]
    fn test_deserialize_message_system() {
        let message = format!(
            r#"{{
                "system": {{
                  "command": "get_access_code",
                  "sequence_id": {},
                  "access_code": "12312312",
                  "result": "success"
                }}
              }}"#,
            2
        );

        let result = serde_json::from_str::<Message>(&message);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Message::System(_)));
    }
}
