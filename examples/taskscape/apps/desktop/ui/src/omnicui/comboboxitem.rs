//! Generated from omnicui/contracts/comboboxitem.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.comboboxitem`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ComboboxitemProps {
    pub context: String,
    pub icon: String,
    pub label: String,
    pub value: String,
    pub id: String,
    pub shortcut: String,
    pub active: bool,
}

impl Default for ComboboxitemProps {
    fn default() -> Self {
        Self {
            context: "palette".to_owned(),
            icon: "".to_owned(),
            label: "".to_owned(),
            value: "".to_owned(),
            id: "".to_owned(),
            shortcut: "".to_owned(),
            active: false,
        }
    }
}

/// The events `ui.comboboxitem` reports.
#[derive(Debug, Clone)]
pub enum ComboboxitemEvent {
    Picked { id: String },
    Highlighted { id: String },
}

impl ComboboxitemEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Picked { id } => ("picked", serde_json::json!({ "id": id })),
            Self::Highlighted { id } => ("highlighted", serde_json::json!({ "id": id })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const INPUT: &str = "ui.comboboxitem.input";
    pub const ROW: &str = "ui.comboboxitem.row";
    pub const ICON: &str = "ui.comboboxitem.icon";
    pub const LABEL: &str = "ui.comboboxitem.label";
    pub const SHORTCUT: &str = "ui.comboboxitem.shortcut";
}
