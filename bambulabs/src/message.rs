//! A message from the printer.

use std::collections::BTreeMap;

use parse_display::{Display, FromStr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    command::{AccessoryType, LedMode, LedNode},
    sequence_id::SequenceId,
};

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
    /// A security message.
    Security(Security),
    /// A liveview message.
    LiveView(LiveView),
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
            Message::Print(print) => Some(print.sequence_id()),
            Message::Info(info) => Some(info.sequence_id()),
            Message::System(system) => Some(system.sequence_id()),
            Message::Security(security) => Some(security.sequence_id()),
            Message::LiveView(live_view) => Some(live_view.sequence_id()),
            Message::Json(_) | Message::Unknown(_) => None,
        }
    }
}

impl From<Print> for Message {
    fn from(print: Print) -> Self {
        Message::Print(print)
    }
}

impl From<Info> for Message {
    fn from(info: Info) -> Self {
        Message::Info(info)
    }
}

impl From<System> for Message {
    fn from(system: System) -> Self {
        Message::System(system)
    }
}

impl From<Security> for Message {
    fn from(security: Security) -> Self {
        Message::Security(security)
    }
}

impl From<LiveView> for Message {
    fn from(live_view: LiveView) -> Self {
        Message::LiveView(live_view)
    }
}

/// A security message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "command")]
pub enum Security {
    /// Get the serial number.
    GetSn(GetSn),
}

impl Security {
    /// Returns the sequence id of the message.
    pub fn sequence_id(&self) -> SequenceId {
        match self {
            Security::GetSn(get_sn) => get_sn.sequence_id.clone(),
        }
    }
}

/// A get serial number message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GetSn {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The serial number.
    pub sn: String,
    /// The address.
    pub address: i64,
    /// The chip sn.
    pub chip_sn: String,
    /// The chip sn length.
    pub chipsn_len: i64,
    /// The length.
    pub length: i64,
    /// The module.
    pub module: String,
    /// The status.
    pub status: String,
    /// The reason for the message.
    pub reason: Option<Reason>,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A liveview message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "command")]
pub enum LiveView {
    /// Initialize the live view.
    Init(Init),
}

impl LiveView {
    /// Returns the sequence id of the message.
    pub fn sequence_id(&self) -> SequenceId {
        match self {
            LiveView::Init(init) => init.sequence_id.clone(),
        }
    }
}

/// An init live view message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Init {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The op protocols.
    pub op_protocols: Vec<OperationProtocol>,
    /// The peer host.
    pub peer_host: String,
    /// The reason for the message.
    pub reason: Option<Reason>,
    /// The result of the command.
    pub result: Result,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// An operation protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OperationProtocol {
    /// The protocol.
    pub protocol: String,
    /// The version.
    pub version: String,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A reason for a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Display, FromStr)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[display(style = "SNAKE_CASE")]
pub enum Reason {
    /// Success.
    #[serde(alias = "success")]
    Success,
    /// Fail.
    #[serde(alias = "fail")]
    Fail,
    /// Some unknown string.
    #[display("{0}")]
    #[serde(untagged)]
    Unknown(String),
}

/// The result of a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Display, FromStr)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[display(style = "SNAKE_CASE")]
pub enum Result {
    /// Success.
    #[serde(alias = "success")]
    Success,
    /// Fail.
    #[serde(alias = "fail")]
    Fail,
}

/// A print command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "command")]
pub enum Print {
    /// Ams control.
    AmsControl(AmsControl),
    /// Ams change filament.
    AmsChangeFilament(AmsChangeFilament),
    /// Calibration.
    Calibration(Calibration),
    /// The status of the print.
    PushStatus(PushStatus),
    /// The gcode line.
    GcodeLine(GcodeLine),
    /// Project file.
    ProjectFile(ProjectFile),
    /// Pause the print.
    Pause(Pause),
    /// Print speed.
    PrintSpeed(PrintSpeed),
    /// Resume the print.
    Resume(Resume),
    /// Stop the print.
    Stop(Stop),
    /// Extrusion calibration get.
    ExtrusionCaliGet(ExtrusionCaliGet),
}

impl Print {
    /// Returns the sequence id of the message.
    pub fn sequence_id(&self) -> SequenceId {
        match self {
            Print::AmsControl(ams_ctrl) => ams_ctrl.sequence_id.clone(),
            Print::AmsChangeFilament(ams_change_filament) => ams_change_filament.sequence_id.clone(),
            Print::Calibration(calibration) => calibration.sequence_id.clone(),
            Print::PushStatus(push_status) => push_status.sequence_id.clone(),
            Print::GcodeLine(gcode_line) => gcode_line.sequence_id.clone(),
            Print::ProjectFile(project_file) => project_file.sequence_id.clone(),
            Print::Pause(pause) => pause.sequence_id.clone(),
            Print::PrintSpeed(print_speed) => print_speed.sequence_id.clone(),
            Print::Resume(resume) => resume.sequence_id.clone(),
            Print::Stop(stop) => stop.sequence_id.clone(),
            Print::ExtrusionCaliGet(extrusion_cali_get) => extrusion_cali_get.sequence_id.clone(),
        }
    }
}

/// An ams control command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AmsControl {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The reason for the message.
    pub reason: Reason,
    /// The result of the command.
    pub result: Result,
    /// The param.
    pub param: Option<String>,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// An ams change filament.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AmsChangeFilament {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The reason for the message.
    pub reason: Option<Reason>,
    /// The result of the command.
    pub result: Result,
    /// The error number.
    pub errorno: i64,
    /// The target temperature.
    pub tar_temp: i64,
    /// The target.
    pub target: i64,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A calibration command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Calibration {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The option.
    pub option: i64,
    /// The reason for the message.
    pub reason: Option<Reason>,
    /// The result of the command.
    pub result: Result,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A gcode line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GcodeLine {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The gcode line.
    pub param: Option<String>,
    /// The reason for the message.
    pub reason: Reason,
    /// The result of the command.
    pub result: Result,
    /// The source.
    pub source: Option<i64>,
    /// The return code.
    pub return_code: Option<String>,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A project file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProjectFile {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The project id.
    pub project_id: String,
    /// The profile id.
    pub profile_id: String,
    /// The task id.
    pub task_id: String,
    /// The subtask id.
    pub subtask_id: String,
    /// The subtask name.
    pub subtask_name: String,
    /// The gcode file.
    pub gcode_file: Option<String>,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A pause command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Pause {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The reason for the message.
    pub reason: Reason,
    /// The result of the command.
    pub result: Result,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A resume command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Resume {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The reason for the message.
    pub reason: Reason,
    /// The result of the command.
    pub result: Result,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A stop command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Stop {
    /// The sequence id.
    pub sequence_id: SequenceId,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// An extrusion calibration get command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExtrusionCaliGet {
    /// The sequence id.
    pub sequence_id: SequenceId,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A print speed command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PrintSpeed {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The reason for the message.
    pub reason: Option<Reason>,
    /// The result of the command.
    pub result: Result,
    /// The param.
    pub param: String,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A push status message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PushStatus {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The aux part fan.
    pub aux_part_fan: Option<bool>,
    /// The upload.
    pub upload: Option<PrintUpload>,
    /// The nozzle diameter.
    pub nozzle_diameter: Option<String>,
    /// The nozzle temperature.
    pub nozzle_temper: Option<f64>,
    /// The nozzle type.
    pub nozzle_type: Option<NozzleType>,
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
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
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
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
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
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
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
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
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
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
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
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
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
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A print lights report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PrintLightsReport {
    /// The node.
    pub node: LedNode,
    /// The mode.
    pub mode: LedMode,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
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
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// An info command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "command")]
pub enum Info {
    /// Get the version.
    GetVersion(GetVersion),
}

impl Info {
    /// Returns the sequence id of the message.
    pub fn sequence_id(&self) -> SequenceId {
        match self {
            Info::GetVersion(get_version) => get_version.sequence_id.clone(),
        }
    }
}

/// A get version message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GetVersion {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The info module.
    pub module: Vec<InfoModule>,
    /// The result of the info command.
    pub result: Option<Result>,
    /// The reason of the info command.
    pub reason: Option<Reason>,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// An info module.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InfoModule {
    /// The module name.
    pub name: String,
    /// The project name.
    pub project_name: Option<String>,
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

/// A system command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "command")]
pub enum System {
    /// Led control.
    Ledctrl(Ledctrl),
    /// Get accessories.
    GetAccessories(GetAccessories),
}

impl System {
    /// Returns the sequence id of the message.
    pub fn sequence_id(&self) -> SequenceId {
        match self {
            System::Ledctrl(ledctrl) => ledctrl.sequence_id.clone(),
            System::GetAccessories(get_accessories) => get_accessories.sequence_id.clone(),
        }
    }
}

/// An led control command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Ledctrl {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The reason for the message.
    pub reason: Option<Reason>,
    /// The result of the command.
    pub result: Result,
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
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A get accessories command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GetAccessories {
    /// The sequence id.
    pub sequence_id: SequenceId,
    /// The reason for the message.
    pub reason: Option<Reason>,
    /// The result of the command.
    pub result: Result,
    /// The accessory type.
    pub accessory_type: AccessoryType,
    /// The aux part fan.
    pub aux_part_fan: bool,
    /// The nozzle diameter.
    pub nozzle_diameter: f64,
    /// The nozzle type.
    pub nozzle_type: NozzleType,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// A nozzle type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NozzleType {
    /// Hardened steel nozzle.
    HardenedSteel,
    /// Stainless steel nozzle.
    StainlessSteel,
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
                    "result":"SUCCESS",
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
                  "command": "ledctrl",
                  "result": "SUCCESS",
                  "led_node": "work_light",
                  "led_mode": "on",
                  "led_on_time": 1000,
                  "led_off_time": 1000,
                  "loop_times": 10,
                  "interval_time": 1000,
                  "sequence_id": {}
                }}
              }}"#,
            2
        );

        let result = serde_json::from_str::<Message>(&message);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Message::System(_)));
    }

    #[test]
    fn test_deserialize_message_system_with_unknown_reason() {
        let message = format!(
            r#"{{
                "system": {{
                  "command": "ledctrl",
                  "result": "fail",
                  "reason": "some other string",
                  "led_node": "chamber_light",
                  "led_mode": "on",
                  "led_on_time": 1000,
                  "led_off_time": 1000,
                  "loop_times": 10,
                  "interval_time": 1000,
                  "sequence_id": {}
                }}
              }}"#,
            2
        );

        let result = serde_json::from_str::<Message>(&message);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Message::System(_)));
    }
}
