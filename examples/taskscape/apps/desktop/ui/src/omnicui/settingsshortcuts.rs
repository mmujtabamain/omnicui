//! Generated from omnicui/contracts/settingsshortcuts.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.settingsshortcuts`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SettingsshortcutsProps {
    pub dataset: String,
}

impl Default for SettingsshortcutsProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
        }
    }
}

/// The events `app.settingsshortcuts` reports.
#[derive(Debug, Clone)]
pub enum SettingsshortcutsEvent {
    BindingChanged { label: String, accel: String },
}

impl SettingsshortcutsEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::BindingChanged { label, accel } => ("bindingChanged", serde_json::json!({ "label": label, "accel": accel })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const DIALOG: &str = "app.settingsshortcuts.dialog";
    pub const TITLE: &str = "app.settingsshortcuts.title";
    pub const CLOSE: &str = "app.settingsshortcuts.close";
    pub const DISCARDALL: &str = "app.settingsshortcuts.discardAll";
    pub const SEARCH: &str = "app.settingsshortcuts.search";
    pub const SEARCHCLEAR: &str = "app.settingsshortcuts.searchClear";
    pub const TABGENERAL: &str = "app.settingsshortcuts.tabGeneral";
    pub const TABSHORTCUTS: &str = "app.settingsshortcuts.tabShortcuts";
    pub const TABABOUT: &str = "app.settingsshortcuts.tabAbout";
    pub const SHORTCUTSMATCHES: &str = "app.settingsshortcuts.shortcutsMatches";
    pub const PANE: &str = "app.settingsshortcuts.pane";
    pub const INTRO: &str = "app.settingsshortcuts.intro";
    pub const GROUPGLOBAL: &str = "app.settingsshortcuts.groupGlobal";
    pub const ROWCAPTUREBAR: &str = "app.settingsshortcuts.rowCaptureBar";
    pub const KEYCAPCAPTUREBAR: &str = "app.settingsshortcuts.keycapCaptureBar";
    pub const KEYCAPSCREENSHOTCAPTURE: &str = "app.settingsshortcuts.keycapScreenshotCapture";
    pub const GROUPMAIN: &str = "app.settingsshortcuts.groupMain";
    pub const DISCARDGROUP: &str = "app.settingsshortcuts.discardGroup";
    pub const ROW: &str = "app.settingsshortcuts.row";
    pub const ROWNEWTASK: &str = "app.settingsshortcuts.rowNewTask";
    pub const ROWNEWLIST: &str = "app.settingsshortcuts.rowNewList";
    pub const ROWNEWPROJECT: &str = "app.settingsshortcuts.rowNewProject";
    pub const ROWUNDO: &str = "app.settingsshortcuts.rowUndo";
    pub const ROWREDO: &str = "app.settingsshortcuts.rowRedo";
    pub const KEYCAP: &str = "app.settingsshortcuts.keycap";
    pub const KEYCAPNEWTASK: &str = "app.settingsshortcuts.keycapNewTask";
    pub const KEYCAPNEWLIST: &str = "app.settingsshortcuts.keycapNewList";
    pub const KEYCAPSEARCH: &str = "app.settingsshortcuts.keycapSearch";
    pub const KEYCAPSWITCHLIST1: &str = "app.settingsshortcuts.keycapSwitchList1";
    pub const KEYCAPPREVTAB: &str = "app.settingsshortcuts.keycapPrevTab";
    pub const KEYCAPPREVPROJECT: &str = "app.settingsshortcuts.keycapPrevProject";
    pub const KEYCAPUNDO: &str = "app.settingsshortcuts.keycapUndo";
    pub const KEYCAPREDO: &str = "app.settingsshortcuts.keycapRedo";
    pub const RESET: &str = "app.settingsshortcuts.reset";
    pub const SHORTCUTERROR: &str = "app.settingsshortcuts.shortcutError";
    pub const NOMATCH: &str = "app.settingsshortcuts.noMatch";
}
