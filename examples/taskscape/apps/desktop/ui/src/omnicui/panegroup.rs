//! Generated from omnicui/contracts/panegroup.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.panegroup`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct PanegroupProps {
    pub width: i64,
    pub direction: String,
    pub split: bool,
    pub asideHidden: bool,
    pub noLists: bool,
    pub autoSaveId: String,
}

impl Default for PanegroupProps {
    fn default() -> Self {
        Self {
            width: 0,
            direction: "row".to_owned(),
            split: false,
            asideHidden: false,
            noLists: false,
            autoSaveId: "".to_owned(),
        }
    }
}

/// The events `ui.panegroup` reports.
#[derive(Debug, Clone)]
pub enum PanegroupEvent {
    LayoutChanged { task: f64, split: f64, aside: f64 },
}

impl PanegroupEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::LayoutChanged { task, split, aside } => ("layoutChanged", serde_json::json!({ "task": task, "split": split, "aside": aside })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TASKPANE: &str = "ui.panegroup.taskPane";
    pub const SPLITPANE: &str = "ui.panegroup.splitPane";
    pub const ASIDE: &str = "ui.panegroup.aside";
    pub const TASKRESIZER: &str = "ui.panegroup.taskResizer";
    pub const ASIDERESIZER: &str = "ui.panegroup.asideResizer";
}
