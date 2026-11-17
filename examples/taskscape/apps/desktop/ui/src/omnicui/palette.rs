//! Generated from omnicui/contracts/palette.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.palette`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct PaletteProps {
    pub dataset: String,
    pub task: String,
}

impl Default for PaletteProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            task: "".to_owned(),
        }
    }
}

/// The events `app.palette` reports.
#[derive(Debug, Clone)]
pub enum PaletteEvent {
    Ran { label: String },
}

impl PaletteEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Ran { label } => ("ran", serde_json::json!({ "label": label })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const SEARCH: &str = "app.palette.search";
    pub const SCRIM: &str = "app.palette.scrim";
    pub const PANEL: &str = "app.palette.panel";
    pub const RESULTS: &str = "app.palette.results";
    pub const ROW: &str = "app.palette.row";
    pub const ROWSWAPPANES: &str = "app.palette.rowSwapPanes";
    pub const ROWSPRINGLAUNCH: &str = "app.palette.rowSpringLaunch";
    pub const ROWDELETE: &str = "app.palette.rowDelete";
    pub const GROUPVIEW: &str = "app.palette.groupView";
    pub const GROUPTASK: &str = "app.palette.groupTask";
    pub const GROUPMANAGE: &str = "app.palette.groupManage";
    pub const GROUPGOLIST: &str = "app.palette.groupGoList";
    pub const GROUPGOPROJECT: &str = "app.palette.groupGoProject";
    pub const EMPTYMESSAGE: &str = "app.palette.emptyMessage";
}
