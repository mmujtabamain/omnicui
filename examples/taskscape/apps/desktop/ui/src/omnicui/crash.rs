//! Generated from omnicui/contracts/crash.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.crash`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct CrashProps {
    pub message: String,
}

impl Default for CrashProps {
    fn default() -> Self {
        Self {
            message: "Something broke".to_owned(),
        }
    }
}

/// The events `app.crash` reports.
#[derive(Debug, Clone)]
pub enum CrashEvent {
}

impl CrashEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const RELOAD: &str = "app.crash.reload";
}
