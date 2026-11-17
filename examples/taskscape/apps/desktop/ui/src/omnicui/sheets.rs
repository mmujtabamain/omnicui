//! Generated from omnicui/contracts/sheets.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.sheets`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SheetsProps {
    pub dataset: String,
    pub sheet: String,
    pub list: String,
    pub project: String,
    pub task: String,
}

impl Default for SheetsProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            sheet: "newList".to_owned(),
            list: "Reading".to_owned(),
            project: "Spring launch".to_owned(),
            task: "Book bike service".to_owned(),
        }
    }
}

/// The events `app.sheets` reports.
#[derive(Debug, Clone)]
pub enum SheetsEvent {
    Answered { action: String, text: String, id: String },
}

impl SheetsEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Answered { action, text, id } => ("answered", serde_json::json!({ "action": action, "text": text, "id": id })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const SCRIM: &str = "app.sheets.scrim";
    pub const CARD: &str = "app.sheets.card";
    pub const GLYPH: &str = "app.sheets.glyph";
    pub const HEADLINE: &str = "app.sheets.headline";
    pub const DETAIL: &str = "app.sheets.detail";
    pub const NAMEFIELD: &str = "app.sheets.nameField";
    pub const ROLL: &str = "app.sheets.roll";
    pub const IMPORTJSON: &str = "app.sheets.importJson";
    pub const CANCEL: &str = "app.sheets.cancel";
    pub const ACCEPT: &str = "app.sheets.accept";
    pub const TOAST: &str = "app.sheets.toast";
    pub const TOASTUNDO: &str = "app.sheets.toastUndo";
    pub const TOASTCLOSE: &str = "app.sheets.toastClose";
}
