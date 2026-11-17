//! Generated from omnicui/contracts/listtablabel.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.listtablabel`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ListtablabelProps {
    pub name: String,
    pub selected: bool,
    pub surface: String,
    pub paneMarker: bool,
    pub closable: bool,
}

impl Default for ListtablabelProps {
    fn default() -> Self {
        Self {
            name: String::new(),
            selected: false,
            surface: "flat".to_owned(),
            paneMarker: false,
            closable: false,
        }
    }
}

/// The events `ui.listtablabel` reports.
#[derive(Debug, Clone)]
pub enum ListtablabelEvent {
    Select {  },
    Close {  },
}

impl ListtablabelEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Select {  } => ("select", serde_json::json!({  })),
            Self::Close {  } => ("close", serde_json::json!({  })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TAB: &str = "ui.listtablabel.tab";
    pub const CLOSE: &str = "ui.listtablabel.close";
    pub const PANEMARKER: &str = "ui.listtablabel.paneMarker";
}
