//! Generated from omnicui/contracts/sectionheader.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.sectionheader`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SectionheaderProps {
    pub label: String,
    pub trailing: String,
    pub active: bool,
    pub restore: String,
    pub restoreLabel: String,
    pub r#as: String,
    pub inset: String,
    pub rule: String,
}

impl Default for SectionheaderProps {
    fn default() -> Self {
        Self {
            label: "".to_owned(),
            trailing: "".to_owned(),
            active: false,
            restore: "none".to_owned(),
            restoreLabel: "".to_owned(),
            r#as: "h3".to_owned(),
            inset: "block".to_owned(),
            rule: "none".to_owned(),
        }
    }
}

/// The events `ui.sectionheader` reports.
#[derive(Debug, Clone)]
pub enum SectionheaderEvent {
    RestoreRequested { kind: String },
}

impl SectionheaderEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::RestoreRequested { kind } => ("restoreRequested", serde_json::json!({ "kind": kind })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TITLE: &str = "ui.sectionheader.title";
    pub const TRAILING: &str = "ui.sectionheader.trailing";
    pub const RULE: &str = "ui.sectionheader.rule";
    pub const CHANGEDDOT: &str = "ui.sectionheader.changedDot";
    pub const RESTORE: &str = "ui.sectionheader.restore";
}
