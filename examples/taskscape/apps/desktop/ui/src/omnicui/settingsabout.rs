//! Generated from omnicui/contracts/settingsabout.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.settingsabout`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SettingsaboutProps {
    pub dataset: String,
    pub version: String,
    pub dataFolder: String,
}

impl Default for SettingsaboutProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            version: "0.1.0".to_owned(),
            dataFolder: "/Users/you/.taskscape".to_owned(),
        }
    }
}

/// The events `app.settingsabout` reports.
#[derive(Debug, Clone)]
pub enum SettingsaboutEvent {
    ResetAll { scope: String },
}

impl SettingsaboutEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::ResetAll { scope } => ("resetAll", serde_json::json!({ "scope": scope })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const WINDOW: &str = "app.settingsabout.window";
    pub const CLOSELIGHT: &str = "app.settingsabout.closeLight";
    pub const MINIMIZELIGHT: &str = "app.settingsabout.minimizeLight";
    pub const FULLSCREENLIGHT: &str = "app.settingsabout.fullscreenLight";
    pub const TITLE: &str = "app.settingsabout.title";
    pub const DISCARDALL: &str = "app.settingsabout.discardAll";
    pub const SEARCH: &str = "app.settingsabout.search";
    pub const SEARCHCLEAR: &str = "app.settingsabout.searchClear";
    pub const NAVGENERAL: &str = "app.settingsabout.navGeneral";
    pub const NAVCAPTURE: &str = "app.settingsabout.navCapture";
    pub const NAVSHORTCUTS: &str = "app.settingsabout.navShortcuts";
    pub const NAVABOUT: &str = "app.settingsabout.navAbout";
    pub const ABOUTMATCHES: &str = "app.settingsabout.aboutMatches";
    pub const PANEL: &str = "app.settingsabout.panel";
    pub const EMPTYSTATE: &str = "app.settingsabout.emptyState";
    pub const APPLICATIONHEADER: &str = "app.settingsabout.applicationHeader";
    pub const RESETHEADER: &str = "app.settingsabout.resetHeader";
    pub const APPNAME: &str = "app.settingsabout.appName";
    pub const APPDESCRIPTION: &str = "app.settingsabout.appDescription";
    pub const VERSION: &str = "app.settingsabout.version";
    pub const DATAFOLDERPATH: &str = "app.settingsabout.dataFolderPath";
    pub const PATHBUTTON: &str = "app.settingsabout.pathButton";
    pub const RESTOREDEFAULTS: &str = "app.settingsabout.restoreDefaults";
    pub const RESTOREDESCRIPTION: &str = "app.settingsabout.restoreDescription";
    pub const RESETBUTTON: &str = "app.settingsabout.resetButton";
    pub const CONFIRMRESET: &str = "app.settingsabout.confirmReset";
    pub const CANCELRESET: &str = "app.settingsabout.cancelReset";
}
