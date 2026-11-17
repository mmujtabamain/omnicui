//! Generated from omnicui/contracts/footer.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.footer`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct FooterProps {
    pub density: String,
    pub tone: String,
    pub edge: String,
    pub drag: String,
}

impl Default for FooterProps {
    fn default() -> Self {
        Self {
            density: "bar".to_owned(),
            tone: "base".to_owned(),
            edge: "top".to_owned(),
            drag: "none".to_owned(),
        }
    }
}

/// The events `ui.footer` reports.
#[derive(Debug, Clone)]
pub enum FooterEvent {
}

impl FooterEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const FOOTER: &str = "ui.footer.footer";
    pub const CANCEL: &str = "ui.footer.cancel";
    pub const ACCEPT: &str = "ui.footer.accept";
}
