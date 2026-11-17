//! Generated from omnicui/contracts/appbar.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.appbar`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct AppbarProps {
    pub dataset: String,
    pub reducedMotion: String,
}

impl Default for AppbarProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            reducedMotion: "off".to_owned(),
        }
    }
}

/// The events `app.appbar` reports.
#[derive(Debug, Clone)]
pub enum AppbarEvent {
    ProjectSelected { name: String },
    ProjectRenameRequested { name: String },
    ProjectDeleteRequested { name: String },
    ProjectCreateRequested { activeProject: String },
    ListCreateRequested { project: String },
    ListSelected { name: String },
    ListRenamed { name: String },
    ListRenameRequested { name: String },
    ListDeleteRequested { name: String },
    ListExportRequested { name: String },
    SplitToggled { active: bool },
    PanesSwapped { left: String },
    PreviewToggled { open: bool },
    TrashOpened { open: bool },
    PaletteOpened { open: bool },
    SettingsOpened { open: bool },
    SignOutRequested { signedIn: bool },
}

impl AppbarEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::ProjectSelected { name } => ("projectSelected", serde_json::json!({ "name": name })),
            Self::ProjectRenameRequested { name } => ("projectRenameRequested", serde_json::json!({ "name": name })),
            Self::ProjectDeleteRequested { name } => ("projectDeleteRequested", serde_json::json!({ "name": name })),
            Self::ProjectCreateRequested { activeProject } => ("projectCreateRequested", serde_json::json!({ "activeProject": activeProject })),
            Self::ListCreateRequested { project } => ("listCreateRequested", serde_json::json!({ "project": project })),
            Self::ListSelected { name } => ("listSelected", serde_json::json!({ "name": name })),
            Self::ListRenamed { name } => ("listRenamed", serde_json::json!({ "name": name })),
            Self::ListRenameRequested { name } => ("listRenameRequested", serde_json::json!({ "name": name })),
            Self::ListDeleteRequested { name } => ("listDeleteRequested", serde_json::json!({ "name": name })),
            Self::ListExportRequested { name } => ("listExportRequested", serde_json::json!({ "name": name })),
            Self::SplitToggled { active } => ("splitToggled", serde_json::json!({ "active": active })),
            Self::PanesSwapped { left } => ("panesSwapped", serde_json::json!({ "left": left })),
            Self::PreviewToggled { open } => ("previewToggled", serde_json::json!({ "open": open })),
            Self::TrashOpened { open } => ("trashOpened", serde_json::json!({ "open": open })),
            Self::PaletteOpened { open } => ("paletteOpened", serde_json::json!({ "open": open })),
            Self::SettingsOpened { open } => ("settingsOpened", serde_json::json!({ "open": open })),
            Self::SignOutRequested { signedIn } => ("signOutRequested", serde_json::json!({ "signedIn": signedIn })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const PROJECTSWITCHER: &str = "app.appbar.projectSwitcher";
    pub const PROJECTMENU: &str = "app.appbar.projectMenu";
    pub const PROJECTROW: &str = "app.appbar.projectRow";
    pub const PROJECTROWSPRING: &str = "app.appbar.projectRowSpring";
    pub const PROJECTRENAME: &str = "app.appbar.projectRename";
    pub const PROJECTDELETE: &str = "app.appbar.projectDelete";
    pub const PROJECTDELETESPRING: &str = "app.appbar.projectDeleteSpring";
    pub const PROJECTNEW: &str = "app.appbar.projectNew";
    pub const LISTTABS: &str = "app.appbar.listTabs";
    pub const LISTTAB: &str = "app.appbar.listTab";
    pub const LISTTABREADING: &str = "app.appbar.listTabReading";
    pub const LISTTABGARDEN: &str = "app.appbar.listTabGarden";
    pub const LISTTABDELETE: &str = "app.appbar.listTabDelete";
    pub const LISTRENAMEINPUT: &str = "app.appbar.listRenameInput";
    pub const NEWLIST: &str = "app.appbar.newList";
    pub const TABMENU: &str = "app.appbar.tabMenu";
    pub const TABMENURENAME: &str = "app.appbar.tabMenuRename";
    pub const TABMENUSPLIT: &str = "app.appbar.tabMenuSplit";
    pub const TABMENUEXPORT: &str = "app.appbar.tabMenuExport";
    pub const TABMENUDELETE: &str = "app.appbar.tabMenuDelete";
    pub const PANEMARKER: &str = "app.appbar.paneMarker";
    pub const SPLITVIEW: &str = "app.appbar.splitView";
    pub const SWAPPANES: &str = "app.appbar.swapPanes";
    pub const PREVIEWTOGGLE: &str = "app.appbar.previewToggle";
    pub const TRASHBUTTON: &str = "app.appbar.trashButton";
    pub const PALETTEBUTTON: &str = "app.appbar.paletteButton";
    pub const SETTINGSBUTTON: &str = "app.appbar.settingsButton";
    pub const ACCOUNTBUTTON: &str = "app.appbar.accountButton";
    pub const ACCOUNTMENU: &str = "app.appbar.accountMenu";
    pub const ACCOUNTEMAIL: &str = "app.appbar.accountEmail";
    pub const ACCOUNTSETTINGS: &str = "app.appbar.accountSettings";
    pub const ACCOUNTSIGNOUT: &str = "app.appbar.accountSignOut";
    pub const REDUCEDMOTIONSTATUS: &str = "app.appbar.reducedMotionStatus";
}
