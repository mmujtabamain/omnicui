//! Generated from omnicui/contracts/row.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.row`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct RowProps {
    pub shape: String,
    pub tone: String,
    pub density: String,
    pub selected: bool,
    pub indent: f64,
}

impl Default for RowProps {
    fn default() -> Self {
        Self {
            shape: "inset".to_owned(),
            tone: "selection".to_owned(),
            density: "md".to_owned(),
            selected: false,
            indent: 0.0,
        }
    }
}

/// The events `ui.row` reports.
#[derive(Debug, Clone)]
pub enum RowEvent {
}

impl RowEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const ROW: &str = "ui.row.row";
    pub const CONTENT: &str = "ui.row.content";
    pub const OPEN: &str = "ui.row.open";
}
