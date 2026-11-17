//! Generated from omnicui/contracts/settingscapture.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.settingscapture`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SettingscaptureProps {
    pub dataset: String,
    pub open: String,
    pub storedScreenshotMode: String,
    pub pathLoading: bool,
}

impl Default for SettingscaptureProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            open: "capture".to_owned(),
            storedScreenshotMode: "fullscreen".to_owned(),
            pathLoading: false,
        }
    }
}

/// The events `app.settingscapture` reports.
#[derive(Debug, Clone)]
pub enum SettingscaptureEvent {
    SettingSaved { key: String, value: String },
    FolderOpenRequested { folder: String },
    WindowClosed { reason: String },
}

impl SettingscaptureEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::SettingSaved { key, value } => ("settingSaved", serde_json::json!({ "key": key, "value": value })),
            Self::FolderOpenRequested { folder } => ("folderOpenRequested", serde_json::json!({ "folder": folder })),
            Self::WindowClosed { reason } => ("windowClosed", serde_json::json!({ "reason": reason })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TITLEBAR: &str = "app.settingscapture.titleBar";
    pub const TITLE: &str = "app.settingscapture.title";
    pub const DISCARDALL: &str = "app.settingscapture.discardAll";
    pub const SIDEBAR: &str = "app.settingscapture.sidebar";
    pub const SEARCHFIELD: &str = "app.settingscapture.searchField";
    pub const CLEARSEARCH: &str = "app.settingscapture.clearSearch";
    pub const SECTIONS: &str = "app.settingscapture.sections";
    pub const SECTIONGENERAL: &str = "app.settingscapture.sectionGeneral";
    pub const SECTIONCAPTURE: &str = "app.settingscapture.sectionCapture";
    pub const SECTIONSHORTCUTS: &str = "app.settingscapture.sectionShortcuts";
    pub const SECTIONABOUT: &str = "app.settingscapture.sectionAbout";
    pub const CAPTUREMATCHCOUNT: &str = "app.settingscapture.captureMatchCount";
    pub const SEARCHEMPTY: &str = "app.settingscapture.searchEmpty";
    pub const CONTENTPANE: &str = "app.settingscapture.contentPane";
    pub const SCREENSHOTSGROUP: &str = "app.settingscapture.screenshotsGroup";
    pub const SCREENSHOTSHEADING: &str = "app.settingscapture.screenshotsHeading";
    pub const SCREENSHOTSDISCARD: &str = "app.settingscapture.screenshotsDiscard";
    pub const CAPTUREMODEROW: &str = "app.settingscapture.captureModeRow";
    pub const CAPTUREMODELABEL: &str = "app.settingscapture.captureModeLabel";
    pub const CAPTUREMODECONTROL: &str = "app.settingscapture.captureModeControl";
    pub const SEGMENTFULLSCREEN: &str = "app.settingscapture.segmentFullScreen";
    pub const SEGMENTREGION: &str = "app.settingscapture.segmentRegion";
    pub const CAPTUREHINT: &str = "app.settingscapture.captureHint";
    pub const SAVEDTOROW: &str = "app.settingscapture.savedToRow";
    pub const SAVEDTOLABEL: &str = "app.settingscapture.savedToLabel";
    pub const SAVEDTODESCRIPTION: &str = "app.settingscapture.savedToDescription";
    pub const PATHBUTTON: &str = "app.settingscapture.pathButton";
    pub const PATHPLACEHOLDER: &str = "app.settingscapture.pathPlaceholder";
    pub const EMPTYTITLE: &str = "app.settingscapture.emptyTitle";
    pub const EMPTYSUBTITLE: &str = "app.settingscapture.emptySubtitle";
}
