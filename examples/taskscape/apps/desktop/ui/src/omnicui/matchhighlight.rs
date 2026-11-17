//! Generated from omnicui/contracts/matchhighlight.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.matchhighlight`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct MatchhighlightProps {
    pub query: String,
    pub text: String,
}

impl Default for MatchhighlightProps {
    fn default() -> Self {
        Self {
            query: "".to_owned(),
            text: "".to_owned(),
        }
    }
}

/// The events `ui.matchhighlight` reports.
#[derive(Debug, Clone)]
pub enum MatchhighlightEvent {
}

impl MatchhighlightEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const HIGHLIGHT: &str = "ui.matchhighlight.highlight";
}
