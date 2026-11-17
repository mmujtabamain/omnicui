//! Generated from omnicui/contracts/segmentedcontrol.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.segmentedcontrol`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SegmentedcontrolProps {
    pub value: String,
    pub segments: String,
    pub icons: String,
    pub groupName: String,
    pub tone: String,
    pub shape: String,
    pub variant: String,
    pub size: String,
    pub disabled: bool,
}

impl Default for SegmentedcontrolProps {
    fn default() -> Self {
        Self {
            value: "all".to_owned(),
            segments: "all=All,active=Active,done=Done".to_owned(),
            icons: "".to_owned(),
            groupName: String::new(),
            tone: "accent".to_owned(),
            shape: "icon".to_owned(),
            variant: "text".to_owned(),
            size: "default".to_owned(),
            disabled: false,
        }
    }
}

/// The events `ui.segmentedcontrol` reports.
#[derive(Debug, Clone)]
pub enum SegmentedcontrolEvent {
    ValueChanged { value: String },
}

impl SegmentedcontrolEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::ValueChanged { value } => ("valueChanged", serde_json::json!({ "value": value })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const GROUP: &str = "ui.segmentedcontrol.group";
    pub const FIRSTSEGMENT: &str = "ui.segmentedcontrol.firstSegment";
    pub const SECONDSEGMENT: &str = "ui.segmentedcontrol.secondSegment";
    pub const THIRDSEGMENT: &str = "ui.segmentedcontrol.thirdSegment";
}
