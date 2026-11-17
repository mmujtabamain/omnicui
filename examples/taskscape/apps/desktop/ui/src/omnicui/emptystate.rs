//! Generated from omnicui/contracts/emptystate.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.emptystate`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct EmptystateProps {
    pub scope: String,
    pub surface: String,
    pub title: String,
    pub subtitle: String,
    pub icon: String,
    pub iconSize: f64,
    pub loading: bool,
    pub actionLabel: String,
    pub className: String,
}

impl Default for EmptystateProps {
    fn default() -> Self {
        Self {
            scope: "list".to_owned(),
            surface: "plain".to_owned(),
            title: "".to_owned(),
            subtitle: "".to_owned(),
            icon: "".to_owned(),
            iconSize: 30.0,
            loading: false,
            actionLabel: "".to_owned(),
            className: "".to_owned(),
        }
    }
}

/// The events `ui.emptystate` reports.
#[derive(Debug, Clone)]
pub enum EmptystateEvent {
}

impl EmptystateEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const ROOT: &str = "ui.emptystate.root";
    pub const TITLE: &str = "ui.emptystate.title";
    pub const SUBTITLE: &str = "ui.emptystate.subtitle";
    pub const ICON: &str = "ui.emptystate.icon";
    pub const LOADINGSTATUS: &str = "ui.emptystate.loadingStatus";
    pub const ACTION: &str = "ui.emptystate.action";
}
