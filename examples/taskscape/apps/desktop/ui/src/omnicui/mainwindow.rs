//! Generated from omnicui/contracts/mainwindow.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.mainwindow`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct MainwindowProps {
    pub dataset: String,
}

impl Default for MainwindowProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
        }
    }
}

/// The events `app.mainwindow` reports.
#[derive(Debug, Clone)]
pub enum MainwindowEvent {
}

impl MainwindowEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const WINDOW: &str = "app.mainwindow.window";
    pub const TITLEBAR: &str = "app.mainwindow.titleBar";
    pub const LISTTABS: &str = "app.mainwindow.listTabs";
    pub const PROJECTSWITCHER: &str = "app.mainwindow.projectSwitcher";
    pub const PROJECTMENUHOME: &str = "app.mainwindow.projectMenuHome";
    pub const PROJECTMENUSPRING: &str = "app.mainwindow.projectMenuSpring";
    pub const PROJECTNEW: &str = "app.mainwindow.projectNew";
    pub const LISTTAB: &str = "app.mainwindow.listTab";
    pub const LISTTABREADING: &str = "app.mainwindow.listTabReading";
    pub const LISTTABGARDEN: &str = "app.mainwindow.listTabGarden";
    pub const LISTTABNEW: &str = "app.mainwindow.listTabNew";
    pub const DELETELIST: &str = "app.mainwindow.deleteList";
    pub const DELETELISTREADING: &str = "app.mainwindow.deleteListReading";
    pub const NEWLIST: &str = "app.mainwindow.newList";
    pub const SPLITVIEW: &str = "app.mainwindow.splitView";
    pub const SWAPPANES: &str = "app.mainwindow.swapPanes";
    pub const PREVIEWTOGGLE: &str = "app.mainwindow.previewToggle";
    pub const COMMANDBUTTON: &str = "app.mainwindow.commandButton";
    pub const TRASH: &str = "app.mainwindow.trash";
    pub const SETTINGSBUTTON: &str = "app.mainwindow.settingsButton";
    pub const ACCOUNTBUTTON: &str = "app.mainwindow.accountButton";
    pub const ACCOUNTMENU: &str = "app.mainwindow.accountMenu";
    pub const ACCOUNTEMAIL: &str = "app.mainwindow.accountEmail";
    pub const ACCOUNTSETTINGS: &str = "app.mainwindow.accountSettings";
    pub const ACCOUNTSIGNOUT: &str = "app.mainwindow.accountSignOut";
    pub const TASKPANE: &str = "app.mainwindow.taskPane";
    pub const SEARCHFIELD: &str = "app.mainwindow.searchField";
    pub const SEARCHOPTIONS: &str = "app.mainwindow.searchOptions";
    pub const SCOPEMENU: &str = "app.mainwindow.scopeMenu";
    pub const SCOPEWHOLEPROJECT: &str = "app.mainwindow.scopeWholeProject";
    pub const GROUPHEADING: &str = "app.mainwindow.groupHeading";
    pub const MATCHROW: &str = "app.mainwindow.matchRow";
    pub const NEWTASK: &str = "app.mainwindow.newTask";
    pub const ROW: &str = "app.mainwindow.row";
    pub const ROWTITLE: &str = "app.mainwindow.rowTitle";
    pub const ROWSELECT: &str = "app.mainwindow.rowSelect";
    pub const ROWMORE: &str = "app.mainwindow.rowMore";
    pub const ROWMENU: &str = "app.mainwindow.rowMenu";
    pub const ROWTITLEEDIT: &str = "app.mainwindow.rowTitleEdit";
    pub const ROWBIKE: &str = "app.mainwindow.rowBike";
    pub const CHECKBIKE: &str = "app.mainwindow.checkBike";
    pub const ROWKEYSTITLE: &str = "app.mainwindow.rowKeysTitle";
    pub const ROWKEYSSELECT: &str = "app.mainwindow.rowKeysSelect";
    pub const ROWGROCERIES: &str = "app.mainwindow.rowGroceries";
    pub const ROWGROCERIESEXPAND: &str = "app.mainwindow.rowGroceriesExpand";
    pub const SUBTASKCOUNT: &str = "app.mainwindow.subtaskCount";
    pub const SUBTASKOATMILK: &str = "app.mainwindow.subtaskOatMilk";
    pub const ROWHEROREWRITE: &str = "app.mainwindow.rowHeroRewrite";
    pub const ATTACHMENTBADGE: &str = "app.mainwindow.attachmentBadge";
    pub const DONECOUNT: &str = "app.mainwindow.doneCount";
    pub const FOOTERPERCENT: &str = "app.mainwindow.footerPercent";
    pub const SORTFILTER: &str = "app.mainwindow.sortFilter";
    pub const SELECTIONCOUNT: &str = "app.mainwindow.selectionCount";
    pub const MULTIINSPECTOR: &str = "app.mainwindow.multiInspector";
    pub const OPENSELECTION: &str = "app.mainwindow.openSelection";
    pub const CLEARSELECTION: &str = "app.mainwindow.clearSelection";
    pub const BULKMARKDONE: &str = "app.mainwindow.bulkMarkDone";
    pub const PREVIEWPANEL: &str = "app.mainwindow.previewPanel";
    pub const PREVIEWEMPTY: &str = "app.mainwindow.previewEmpty";
    pub const PREVIEWTITLE: &str = "app.mainwindow.previewTitle";
    pub const DRAFTINSPECTOR: &str = "app.mainwindow.draftInspector";
    pub const PREVIEWDIVIDER: &str = "app.mainwindow.previewDivider";
    pub const TRASHPANE: &str = "app.mainwindow.trashPane";
    pub const TRASHHEADING: &str = "app.mainwindow.trashHeading";
    pub const TRASHCLOSE: &str = "app.mainwindow.trashClose";
    pub const FILTERPANEL: &str = "app.mainwindow.filterPanel";
    pub const FILTERHEADING: &str = "app.mainwindow.filterHeading";
    pub const SPLITDIVIDER: &str = "app.mainwindow.splitDivider";
    pub const SPLITPANE: &str = "app.mainwindow.splitPane";
    pub const SPLITSEARCH: &str = "app.mainwindow.splitSearch";
    pub const SPLITCLOSE: &str = "app.mainwindow.splitClose";
    pub const SPLITROW: &str = "app.mainwindow.splitRow";
    pub const SPLITFOOTERDONECOUNT: &str = "app.mainwindow.splitFooterDoneCount";
    pub const NOLISTSTITLE: &str = "app.mainwindow.noListsTitle";
    pub const NOLISTSSUBTITLE: &str = "app.mainwindow.noListsSubtitle";
    pub const DIALOG: &str = "app.mainwindow.dialog";
    pub const DIALOGHEADING: &str = "app.mainwindow.dialogHeading";
    pub const DIALOGNAME: &str = "app.mainwindow.dialogName";
    pub const DIALOGCREATE: &str = "app.mainwindow.dialogCreate";
    pub const DIALOGCANCEL: &str = "app.mainwindow.dialogCancel";
    pub const DIALOGDELETE: &str = "app.mainwindow.dialogDelete";
    pub const DIALOGMOVETOTRASH: &str = "app.mainwindow.dialogMoveToTrash";
    pub const DIALOGDETAIL: &str = "app.mainwindow.dialogDetail";
    pub const TOAST: &str = "app.mainwindow.toast";
    pub const TOASTUNDO: &str = "app.mainwindow.toastUndo";
    pub const COMMANDPALETTE: &str = "app.mainwindow.commandPalette";
    pub const COMMANDINPUT: &str = "app.mainwindow.commandInput";
    pub const PALETTEOPENTRASH: &str = "app.mainwindow.paletteOpenTrash";
    pub const SETTINGSDIALOG: &str = "app.mainwindow.settingsDialog";
    pub const SETTINGSHEADING: &str = "app.mainwindow.settingsHeading";
    pub const SETTINGSCLOSE: &str = "app.mainwindow.settingsClose";
}
