//! Generated from omnicui/contracts/contextmenus.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.contextmenus`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ContextmenusProps {
    pub dataset: String,
}

impl Default for ContextmenusProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
        }
    }
}

/// The events `app.contextmenus` reports.
#[derive(Debug, Clone)]
pub enum ContextmenusEvent {
}

impl ContextmenusEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const PROJECTSWITCH: &str = "app.contextmenus.projectSwitch";
    pub const PROJECTMENU: &str = "app.contextmenus.projectMenu";
    pub const PROJECTHOME: &str = "app.contextmenus.projectHome";
    pub const PROJECTSPRINGLAUNCH: &str = "app.contextmenus.projectSpringLaunch";
    pub const PROJECTLAST: &str = "app.contextmenus.projectLast";
    pub const PROJECTNEW: &str = "app.contextmenus.projectNew";
    pub const PROJECTRENAMESPRING: &str = "app.contextmenus.projectRenameSpring";
    pub const PROJECTDELETESPRING: &str = "app.contextmenus.projectDeleteSpring";
    pub const FIRSTTAB: &str = "app.contextmenus.firstTab";
    pub const TABREADING: &str = "app.contextmenus.tabReading";
    pub const TABGARDEN: &str = "app.contextmenus.tabGarden";
    pub const TABCLOSEREADING: &str = "app.contextmenus.tabCloseReading";
    pub const TABMOREREADING: &str = "app.contextmenus.tabMoreReading";
    pub const MOREGROCERIES: &str = "app.contextmenus.moreGroceries";
    pub const MOREPRESCRIPTION: &str = "app.contextmenus.morePrescription";
    pub const MOREBIKESERVICE: &str = "app.contextmenus.moreBikeService";
    pub const MORECUTKEY: &str = "app.contextmenus.moreCutKey";
    pub const ROWPRESCRIPTION: &str = "app.contextmenus.rowPrescription";
    pub const ROWBIKESERVICE: &str = "app.contextmenus.rowBikeService";
    pub const ROWEGGS: &str = "app.contextmenus.rowEggs";
    pub const SUBTASKTITLE: &str = "app.contextmenus.subtaskTitle";
    pub const SUBTASKINPUT: &str = "app.contextmenus.subtaskInput";
    pub const TITLEINPUT: &str = "app.contextmenus.titleInput";
    pub const ROWMENU: &str = "app.contextmenus.rowMenu";
    pub const MOVESUBMENU: &str = "app.contextmenus.moveSubmenu";
    pub const MENUCOPY: &str = "app.contextmenus.menuCopy";
    pub const MENUADDSUBTASK: &str = "app.contextmenus.menuAddSubtask";
    pub const MENURENAME: &str = "app.contextmenus.menuRename";
    pub const MENUMOVETOLIST: &str = "app.contextmenus.menuMoveToList";
    pub const MENUDELETE: &str = "app.contextmenus.menuDelete";
    pub const MOVEREADING: &str = "app.contextmenus.moveReading";
    pub const MOVEGARDEN: &str = "app.contextmenus.moveGarden";
    pub const MOVEERRANDS: &str = "app.contextmenus.moveErrands";
    pub const TABMENU: &str = "app.contextmenus.tabMenu";
    pub const MENUTABRENAME: &str = "app.contextmenus.menuTabRename";
    pub const MENUTABSPLIT: &str = "app.contextmenus.menuTabSplit";
    pub const MENUTABEXPORT: &str = "app.contextmenus.menuTabExport";
    pub const MENUTABDELETE: &str = "app.contextmenus.menuTabDelete";
    pub const SHEETHEADLINE: &str = "app.contextmenus.sheetHeadline";
    pub const SHEETDETAIL: &str = "app.contextmenus.sheetDetail";
    pub const SHEETINPUT: &str = "app.contextmenus.sheetInput";
    pub const SHEETACCEPT: &str = "app.contextmenus.sheetAccept";
    pub const SHEETCANCEL: &str = "app.contextmenus.sheetCancel";
    pub const TOAST: &str = "app.contextmenus.toast";
    pub const TOASTUNDO: &str = "app.contextmenus.toastUndo";
    pub const TOASTDISMISS: &str = "app.contextmenus.toastDismiss";
}
