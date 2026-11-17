//! Generated from omnicui/contracts/pane.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.pane`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct PaneProps {
    pub slot: String,
    pub edge: String,
    pub fill: String,
    pub r#as: String,
    pub ratio: f64,
    pub previewWidth: f64,
    pub defaultSize: f64,
    pub minSize: f64,
    pub maxSize: f64,
    pub shrink: bool,
    pub open: bool,
    pub width: f64,
}

impl Default for PaneProps {
    fn default() -> Self {
        Self {
            slot: "primary".to_owned(),
            edge: "none".to_owned(),
            fill: "none".to_owned(),
            r#as: "div".to_owned(),
            ratio: 0.5,
            previewWidth: 320.0,
            defaultSize: 100.0,
            minSize: 0.0,
            maxSize: 0.0,
            shrink: false,
            open: true,
            width: 0.0,
        }
    }
}

/// The events `ui.pane` reports.
#[derive(Debug, Clone)]
pub enum PaneEvent {
}

impl PaneEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const PANE: &str = "ui.pane.pane";
}
