//! Generated from contracts/counter.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.counter`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct CounterProps {
    pub start: i64,
}

impl Default for CounterProps {
    fn default() -> Self {
        Self {
            start: 0,
        }
    }
}

/// The events `ui.counter` reports.
#[derive(Debug, Clone)]
pub enum CounterEvent {
    Changed { count: i64 },
}

impl CounterEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Changed { count } => ("changed", serde_json::json!({ "count": count })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const VALUE: &str = "ui.counter.value";
    pub const INCREMENT: &str = "ui.counter.increment";
}
