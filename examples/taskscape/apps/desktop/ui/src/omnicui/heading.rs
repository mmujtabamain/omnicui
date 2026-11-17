//! Generated from omnicui/contracts/heading.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.heading`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct HeadingProps {
    pub r#as: String,
    pub size: String,
    pub overflow: String,
    pub content: String,
}

impl Default for HeadingProps {
    fn default() -> Self {
        Self {
            r#as: "h2".to_owned(),
            size: "sm".to_owned(),
            overflow: "truncate".to_owned(),
            content: "Title".to_owned(),
        }
    }
}

/// The events `ui.heading` reports.
#[derive(Debug, Clone)]
pub enum HeadingEvent {
}

impl HeadingEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TITLE: &str = "ui.heading.title";
}
