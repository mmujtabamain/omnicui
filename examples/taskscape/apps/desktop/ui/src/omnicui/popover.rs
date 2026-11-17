//! Generated from omnicui/contracts/popover.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.popover`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct PopoverProps {
    pub maxItems: i64,
    pub anchorX: f64,
    pub anchorY: f64,
}

impl Default for PopoverProps {
    fn default() -> Self {
        Self {
            maxItems: 6,
            anchorX: 0.0,
            anchorY: 0.0,
        }
    }
}

/// The events `ui.popover` reports.
#[derive(Debug, Clone)]
pub enum PopoverEvent {
    Picked { value: String },
    Escaped {  },
}

impl PopoverEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Picked { value } => ("picked", serde_json::json!({ "value": value })),
            Self::Escaped {  } => ("escaped", serde_json::json!({  })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const EDITOR: &str = "ui.popover.editor";
    pub const LIST: &str = "ui.popover.list";
    pub const ROW1: &str = "ui.popover.row1";
    pub const ROW2: &str = "ui.popover.row2";
    pub const ROW3: &str = "ui.popover.row3";
    pub const ROW4: &str = "ui.popover.row4";
    pub const ROW5: &str = "ui.popover.row5";
    pub const ROW6: &str = "ui.popover.row6";
}
