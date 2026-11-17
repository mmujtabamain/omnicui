//! Generated from omnicui/contracts/settingswindow.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.settingswindow`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SettingswindowProps {
    pub dataset: String,
    pub motionSignal: String,
    pub storedTheme: String,
}

impl Default for SettingswindowProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            motionSignal: "none".to_owned(),
            storedTheme: "none".to_owned(),
        }
    }
}

/// The events `app.settingswindow` reports.
#[derive(Debug, Clone)]
pub enum SettingswindowEvent {
    SettingChanged { key: String, value: String },
    WindowClosed { reason: String },
}

impl SettingswindowEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::SettingChanged { key, value } => ("settingChanged", serde_json::json!({ "key": key, "value": value })),
            Self::WindowClosed { reason } => ("windowClosed", serde_json::json!({ "reason": reason })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TITLE: &str = "app.settingswindow.title";
    pub const DISCARDALL: &str = "app.settingswindow.discardAll";
    pub const SIDEBAR: &str = "app.settingswindow.sidebar";
    pub const TABLIST: &str = "app.settingswindow.tabList";
    pub const SEARCH: &str = "app.settingswindow.search";
    pub const SEARCHCLEAR: &str = "app.settingswindow.searchClear";
    pub const TABGENERAL: &str = "app.settingswindow.tabGeneral";
    pub const TABCAPTURE: &str = "app.settingswindow.tabCapture";
    pub const TABSHORTCUTS: &str = "app.settingswindow.tabShortcuts";
    pub const TABABOUT: &str = "app.settingswindow.tabAbout";
    pub const GENERALMATCHES: &str = "app.settingswindow.generalMatches";
    pub const CONTENT: &str = "app.settingswindow.content";
    pub const EMPTYSTATE: &str = "app.settingswindow.emptyState";
    pub const EMPTYHINT: &str = "app.settingswindow.emptyHint";
    pub const APPEARANCEHEADING: &str = "app.settingswindow.appearanceHeading";
    pub const APPEARANCEDISCARD: &str = "app.settingswindow.appearanceDiscard";
    pub const APPEARANCEGROUP: &str = "app.settingswindow.appearanceGroup";
    pub const THEMEROW: &str = "app.settingswindow.themeRow";
    pub const THEMETITLE: &str = "app.settingswindow.themeTitle";
    pub const THEMEHINT: &str = "app.settingswindow.themeHint";
    pub const THEMESYSTEM: &str = "app.settingswindow.themeSystem";
    pub const THEMELIGHT: &str = "app.settingswindow.themeLight";
    pub const THEMEDARK: &str = "app.settingswindow.themeDark";
    pub const REDUCEDROW: &str = "app.settingswindow.reducedRow";
    pub const REDUCEDTITLE: &str = "app.settingswindow.reducedTitle";
    pub const REDUCEDHINT: &str = "app.settingswindow.reducedHint";
    pub const REDUCEDSYSTEM: &str = "app.settingswindow.reducedSystem";
    pub const REDUCEDON: &str = "app.settingswindow.reducedOn";
    pub const REDUCEDOFF: &str = "app.settingswindow.reducedOff";
    pub const LOWPOWERNOTE: &str = "app.settingswindow.lowPowerNote";
    pub const SYSTEMNOTE: &str = "app.settingswindow.systemNote";
    pub const TASKSHEADING: &str = "app.settingswindow.tasksHeading";
    pub const TASKSDISCARD: &str = "app.settingswindow.tasksDiscard";
    pub const TASKSGROUP: &str = "app.settingswindow.tasksGroup";
    pub const SHOWCOMPLETEDROW: &str = "app.settingswindow.showCompletedRow";
    pub const SHOWCOMPLETEDTITLE: &str = "app.settingswindow.showCompletedTitle";
    pub const SHOWCOMPLETEDHINT: &str = "app.settingswindow.showCompletedHint";
    pub const SHOWCOMPLETED: &str = "app.settingswindow.showCompleted";
    pub const CAPTUREMODELABEL: &str = "app.settingswindow.captureModeLabel";
    pub const SHORTCUTSNEWTASK: &str = "app.settingswindow.shortcutsNewTask";
    pub const ABOUTAPPNAME: &str = "app.settingswindow.aboutAppName";
}
