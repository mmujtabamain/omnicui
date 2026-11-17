//! Generated from omnicui/contracts/titlebar.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.titlebar`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TitlebarProps {
    pub scope: String,
    pub controls: String,
    pub title: String,
    pub titleAs: String,
    pub projectName: String,
    pub windowFocused: bool,
    pub projectMenuOpen: bool,
    pub scrolled: bool,
    pub recording: bool,
    pub withProjectSwitcher: bool,
    pub withTabs: bool,
    pub withTrailing: bool,
}

impl Default for TitlebarProps {
    fn default() -> Self {
        Self {
            scope: "window".to_owned(),
            controls: "none".to_owned(),
            title: "".to_owned(),
            titleAs: "div".to_owned(),
            projectName: "".to_owned(),
            windowFocused: true,
            projectMenuOpen: false,
            scrolled: false,
            recording: false,
            withProjectSwitcher: false,
            withTabs: false,
            withTrailing: false,
        }
    }
}

/// The events `ui.titlebar` reports.
#[derive(Debug, Clone)]
pub enum TitlebarEvent {
    ProjectMenuChanged { open: bool },
    MaximizeToggled { maximized: bool },
}

impl TitlebarEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::ProjectMenuChanged { open } => ("projectMenuChanged", serde_json::json!({ "open": open })),
            Self::MaximizeToggled { maximized } => ("maximizeToggled", serde_json::json!({ "maximized": maximized })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const BAR: &str = "ui.titlebar.bar";
    pub const TITLE: &str = "ui.titlebar.title";
    pub const CONTROLS: &str = "ui.titlebar.controls";
    pub const CLOSE: &str = "ui.titlebar.close";
    pub const MINIMIZE: &str = "ui.titlebar.minimize";
    pub const MAXIMIZE: &str = "ui.titlebar.maximize";
    pub const PROJECTSWITCHER: &str = "ui.titlebar.projectSwitcher";
    pub const PROJECTMENU: &str = "ui.titlebar.projectMenu";
    pub const TABS: &str = "ui.titlebar.tabs";
    pub const TRAILING: &str = "ui.titlebar.trailing";
    pub const RECORDINGSTATUS: &str = "ui.titlebar.recordingStatus";
    pub const CONTENTSLOT: &str = "ui.titlebar.contentSlot";
}
