//! Generated from omnicui/contracts/menu.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.menu`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct MenuProps {
    pub open: bool,
    pub placement: String,
    pub width: String,
    pub offset: f64,
    pub pointX: f64,
    pub pointY: f64,
    pub closeOnSelect: bool,
    pub ariaLabel: String,
}

impl Default for MenuProps {
    fn default() -> Self {
        Self {
            open: false,
            placement: "cursor".to_owned(),
            width: "content".to_owned(),
            offset: 4.0,
            pointX: 0.0,
            pointY: 0.0,
            closeOnSelect: true,
            ariaLabel: "Menu".to_owned(),
        }
    }
}

/// The events `ui.menu` reports.
#[derive(Debug, Clone)]
pub enum MenuEvent {
    OpenChange { open: bool, reason: String },
    Pick { id: String },
}

impl MenuEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::OpenChange { open, reason } => ("openChange", serde_json::json!({ "open": open, "reason": reason })),
            Self::Pick { id } => ("pick", serde_json::json!({ "id": id })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const PANEL: &str = "ui.menu.panel";
    pub const TRIGGER: &str = "ui.menu.trigger";
    pub const FIRST: &str = "ui.menu.first";
    pub const SECOND: &str = "ui.menu.second";
    pub const SEPARATOR: &str = "ui.menu.separator";
    pub const DISABLEDROW: &str = "ui.menu.disabledRow";
    pub const SUBMENU: &str = "ui.menu.submenu";
    pub const SUBMENUPANEL: &str = "ui.menu.submenuPanel";
    pub const SUBMENUFIRST: &str = "ui.menu.submenuFirst";
    pub const CHOICE: &str = "ui.menu.choice";
    pub const LAST: &str = "ui.menu.last";
    pub const OUTSIDE: &str = "ui.menu.outside";
}
