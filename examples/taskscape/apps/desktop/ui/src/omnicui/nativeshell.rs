//! Generated from omnicui/contracts/nativeshell.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.nativeshell`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct NativeshellProps {
    pub dataset: String,
    pub open: String,
    pub boot: String,
    pub focus: String,
    pub barState: String,
    pub lastProject: String,
    pub lastList: String,
    pub captureMode: String,
    pub captureHotkey: String,
}

impl Default for NativeshellProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            open: "main".to_owned(),
            boot: "done".to_owned(),
            focus: "main".to_owned(),
            barState: "hidden".to_owned(),
            lastProject: "".to_owned(),
            lastList: "".to_owned(),
            captureMode: "fullScreen".to_owned(),
            captureHotkey: "bound".to_owned(),
        }
    }
}

/// The events `app.nativeshell` reports.
#[derive(Debug, Clone)]
pub enum NativeshellEvent {
    ShellCommand { action: String },
    CaptureSubmitted { title: String },
    Quit { code: i64 },
    Remembered { list: String, project: String },
}

impl NativeshellEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::ShellCommand { action } => ("shellCommand", serde_json::json!({ "action": action })),
            Self::CaptureSubmitted { title } => ("captureSubmitted", serde_json::json!({ "title": title })),
            Self::Quit { code } => ("quit", serde_json::json!({ "code": code })),
            Self::Remembered { list, project } => ("remembered", serde_json::json!({ "list": list, "project": project })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const BOOTVEIL: &str = "app.nativeshell.bootVeil";
    pub const BOOTBOX: &str = "app.nativeshell.bootBox";
    pub const MAINWINDOW: &str = "app.nativeshell.mainWindow";
    pub const MENUBAR: &str = "app.nativeshell.menuBar";
    pub const APPMENU: &str = "app.nativeshell.appMenu";
    pub const MENUEDIT: &str = "app.nativeshell.menuEdit";
    pub const MENULIST: &str = "app.nativeshell.menuList";
    pub const MENUIMPORTLIST: &str = "app.nativeshell.menuImportList";
    pub const MENUEXPORTLIST: &str = "app.nativeshell.menuExportList";
    pub const PROJECTSWITCHER: &str = "app.nativeshell.projectSwitcher";
    pub const LISTTAB: &str = "app.nativeshell.listTab";
    pub const OPENLISTLABEL: &str = "app.nativeshell.openListLabel";
    pub const CAPTUREBAR: &str = "app.nativeshell.captureBar";
    pub const CAPTURETITLE: &str = "app.nativeshell.captureTitle";
    pub const CAPTURETARGET: &str = "app.nativeshell.captureTarget";
    pub const TRAYTOOLTIP: &str = "app.nativeshell.trayTooltip";
    pub const TRAYOPEN: &str = "app.nativeshell.trayOpen";
    pub const TRAYQUIT: &str = "app.nativeshell.trayQuit";
}
