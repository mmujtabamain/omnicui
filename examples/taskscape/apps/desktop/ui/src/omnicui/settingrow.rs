//! Generated from omnicui/contracts/settingrow.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.settingrow`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SettingrowProps {
    pub title: String,
    pub description: String,
    pub footnote: String,
    pub footnoteTone: String,
    pub layout: String,
    pub size: String,
    pub control: String,
    pub checked: bool,
    pub choices: String,
    pub choice: String,
    pub icon: bool,
    pub value: String,
    pub recording: bool,
    pub searchOnly: bool,
    pub tone: String,
}

impl Default for SettingrowProps {
    fn default() -> Self {
        Self {
            title: "".to_owned(),
            description: "".to_owned(),
            footnote: "".to_owned(),
            footnoteTone: "note".to_owned(),
            layout: "inline".to_owned(),
            size: "sm".to_owned(),
            control: "none".to_owned(),
            checked: false,
            choices: "System|Light|Dark".to_owned(),
            choice: "".to_owned(),
            icon: false,
            value: "".to_owned(),
            recording: false,
            searchOnly: false,
            tone: "primary".to_owned(),
        }
    }
}

/// The events `ui.settingrow` reports.
#[derive(Debug, Clone)]
pub enum SettingrowEvent {
    Changed { value: String },
    Toggled { checked: bool },
}

impl SettingrowEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Changed { value } => ("changed", serde_json::json!({ "value": value })),
            Self::Toggled { checked } => ("toggled", serde_json::json!({ "checked": checked })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const ROW: &str = "ui.settingrow.row";
    pub const TITLE: &str = "ui.settingrow.title";
    pub const DESCRIPTION: &str = "ui.settingrow.description";
    pub const ICON: &str = "ui.settingrow.icon";
    pub const CONTROL: &str = "ui.settingrow.control";
    pub const TOGGLE: &str = "ui.settingrow.toggle";
    pub const SEGMENTONE: &str = "ui.settingrow.segmentOne";
    pub const SEGMENTTWO: &str = "ui.settingrow.segmentTwo";
    pub const SEGMENTTHREE: &str = "ui.settingrow.segmentThree";
    pub const BUTTON: &str = "ui.settingrow.button";
    pub const FOOTNOTE: &str = "ui.settingrow.footnote";
    pub const REBINDERROR: &str = "ui.settingrow.rebindError";
    pub const VALUE: &str = "ui.settingrow.value";
}
