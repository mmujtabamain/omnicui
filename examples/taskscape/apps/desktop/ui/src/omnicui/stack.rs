//! Generated from omnicui/contracts/stack.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.stack`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct StackProps {
    pub direction: String,
    pub justify: String,
    pub align: String,
    pub grow: String,
    pub gap: String,
    pub dragRegion: bool,
    pub motion: String,
    pub asChild: bool,
}

impl Default for StackProps {
    fn default() -> Self {
        Self {
            direction: "row".to_owned(),
            justify: "start".to_owned(),
            align: "stretch".to_owned(),
            grow: "none".to_owned(),
            gap: "none".to_owned(),
            dragRegion: false,
            motion: "none".to_owned(),
            asChild: false,
        }
    }
}

/// The events `ui.stack` reports.
#[derive(Debug, Clone)]
pub enum StackEvent {
}

impl StackEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const ROOT: &str = "ui.stack.root";
    pub const FIRST: &str = "ui.stack.first";
    pub const SECOND: &str = "ui.stack.second";
}
