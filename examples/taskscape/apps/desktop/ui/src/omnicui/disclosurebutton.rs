//! Generated from omnicui/contracts/disclosurebutton.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.disclosurebutton`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct DisclosurebuttonProps {
    pub expanded: bool,
    pub hidden: bool,
}

impl Default for DisclosurebuttonProps {
    fn default() -> Self {
        Self {
            expanded: false,
            hidden: false,
        }
    }
}

/// The events `ui.disclosurebutton` reports.
#[derive(Debug, Clone)]
pub enum DisclosurebuttonEvent {
    ExpandedChanged { expanded: bool },
    RowOpened {  },
}

impl DisclosurebuttonEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::ExpandedChanged { expanded } => ("expandedChanged", serde_json::json!({ "expanded": expanded })),
            Self::RowOpened {  } => ("rowOpened", serde_json::json!({  })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const DISCLOSURE: &str = "ui.disclosurebutton.disclosure";
    pub const ROW: &str = "ui.disclosurebutton.row";
}
