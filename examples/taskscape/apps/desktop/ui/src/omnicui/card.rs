//! Generated from omnicui/contracts/card.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.card`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct CardProps {
    pub tone: String,
    pub border: String,
    pub radius: String,
    pub context: String,
    pub divided: bool,
    pub asChild: bool,
    pub label: String,
    pub breadcrumb: String,
    pub hint: String,
    pub changed: bool,
    pub discardable: bool,
    pub note: bool,
}

impl Default for CardProps {
    fn default() -> Self {
        Self {
            tone: "surface-2".to_owned(),
            border: "solid".to_owned(),
            radius: "panel".to_owned(),
            context: "page".to_owned(),
            divided: false,
            asChild: false,
            label: "".to_owned(),
            breadcrumb: "".to_owned(),
            hint: "".to_owned(),
            changed: false,
            discardable: false,
            note: false,
        }
    }
}

/// The events `ui.card` reports.
#[derive(Debug, Clone)]
pub enum CardEvent {
    Discarded { label: String },
}

impl CardEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Discarded { label } => ("discarded", serde_json::json!({ "label": label })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const CONTAINER: &str = "ui.card.container";
    pub const HEADING: &str = "ui.card.heading";
    pub const HINT: &str = "ui.card.hint";
    pub const CHANGEDOT: &str = "ui.card.changeDot";
    pub const DISCARD: &str = "ui.card.discard";
    pub const EDITOR: &str = "ui.card.editor";
    pub const EDITORTOOLBAR: &str = "ui.card.editorToolbar";
}
