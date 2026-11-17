//! Generated from omnicui/contracts/keycap.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.keycap`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct KeycapProps {
    pub mode: String,
    pub binding: String,
    pub accel: String,
    pub actionName: String,
    pub prompt: String,
    pub unboundLabel: String,
    pub reserved: String,
    pub scope: String,
    pub disabled: bool,
}

impl Default for KeycapProps {
    fn default() -> Self {
        Self {
            mode: "idle".to_owned(),
            binding: "default".to_owned(),
            accel: "".to_owned(),
            actionName: "".to_owned(),
            prompt: "Type shortcut…".to_owned(),
            unboundLabel: "None".to_owned(),
            reserved: "".to_owned(),
            scope: "app".to_owned(),
            disabled: false,
        }
    }
}

/// The events `ui.keycap` reports.
#[derive(Debug, Clone)]
pub enum KeycapEvent {
    StartRecording {  },
    Capture { accel: String },
    Cancel {  },
    Reset {  },
    Refused { accel: String },
}

impl KeycapEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::StartRecording {  } => ("startRecording", serde_json::json!({  })),
            Self::Capture { accel } => ("capture", serde_json::json!({ "accel": accel })),
            Self::Cancel {  } => ("cancel", serde_json::json!({  })),
            Self::Reset {  } => ("reset", serde_json::json!({  })),
            Self::Refused { accel } => ("refused", serde_json::json!({ "accel": accel })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const GROUP: &str = "ui.keycap.group";
    pub const KEYCAP: &str = "ui.keycap.keycap";
    pub const RESET: &str = "ui.keycap.reset";
    pub const ANNOUNCEMENT: &str = "ui.keycap.announcement";
}
