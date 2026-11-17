//! Generated from omnicui/contracts/menuitem.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.menuitem`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct MenuitemProps {
    pub variant: String,
    pub tone: String,
    pub selection: String,
    pub trailing: String,
    pub size: String,
    pub icon: String,
    pub label: String,
    pub truncate: bool,
    pub checked: bool,
    pub disabled: bool,
    pub value: String,
    pub rowActions: bool,
}

impl Default for MenuitemProps {
    fn default() -> Self {
        Self {
            variant: "item".to_owned(),
            tone: "default".to_owned(),
            selection: "action".to_owned(),
            trailing: "none".to_owned(),
            size: "default".to_owned(),
            icon: "".to_owned(),
            label: "Item".to_owned(),
            truncate: false,
            checked: false,
            disabled: false,
            value: "".to_owned(),
            rowActions: false,
        }
    }
}

/// The events `ui.menuitem` reports.
#[derive(Debug, Clone)]
pub enum MenuitemEvent {
    Activated { value: String, label: String },
    SubmenuToggled { open: bool },
    RenamePressed { label: String },
    DeletePressed { label: String },
}

impl MenuitemEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Activated { value, label } => ("activated", serde_json::json!({ "value": value, "label": label })),
            Self::SubmenuToggled { open } => ("submenuToggled", serde_json::json!({ "open": open })),
            Self::RenamePressed { label } => ("renamePressed", serde_json::json!({ "label": label })),
            Self::DeletePressed { label } => ("deletePressed", serde_json::json!({ "label": label })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const ROW: &str = "ui.menuitem.row";
    pub const LABEL: &str = "ui.menuitem.label";
    pub const ICON: &str = "ui.menuitem.icon";
    pub const CHECK: &str = "ui.menuitem.check";
    pub const CHEVRON: &str = "ui.menuitem.chevron";
    pub const SUBMENU: &str = "ui.menuitem.submenu";
    pub const GROUP: &str = "ui.menuitem.group";
    pub const RENAMEACTION: &str = "ui.menuitem.renameAction";
    pub const DELETEACTION: &str = "ui.menuitem.deleteAction";
    pub const PREVIOUS: &str = "ui.menuitem.previous";
    pub const SIBLING: &str = "ui.menuitem.sibling";
    pub const DISABLEDSIBLING: &str = "ui.menuitem.disabledSibling";
}
