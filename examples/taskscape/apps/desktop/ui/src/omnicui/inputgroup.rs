//! Generated from omnicui/contracts/inputgroup.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.inputgroup`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct InputgroupProps {
    pub shape: String,
    pub tone: String,
    pub size: String,
    pub suffix: String,
    pub focusInputOnPress: bool,
    pub disabled: bool,
    pub sheet: bool,
}

impl Default for InputgroupProps {
    fn default() -> Self {
        Self {
            shape: "well".to_owned(),
            tone: "field".to_owned(),
            size: "md".to_owned(),
            suffix: "".to_owned(),
            focusInputOnPress: true,
            disabled: false,
            sheet: false,
        }
    }
}

/// The events `ui.inputgroup` reports.
#[derive(Debug, Clone)]
pub enum InputgroupEvent {
}

impl InputgroupEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const WELL: &str = "ui.inputgroup.well";
    pub const LEADING: &str = "ui.inputgroup.leading";
    pub const INPUT: &str = "ui.inputgroup.input";
    pub const SUFFIX: &str = "ui.inputgroup.suffix";
    pub const TRAILING: &str = "ui.inputgroup.trailing";
    pub const CLEAR: &str = "ui.inputgroup.clear";
    pub const ROLL: &str = "ui.inputgroup.roll";
}
