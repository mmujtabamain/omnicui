//! Generated from omnicui/contracts/filterpanel.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.filterpanel`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct FilterpanelProps {
    pub dataset: String,
    pub panel: String,
    pub showCompleted: bool,
}

impl Default for FilterpanelProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            panel: "open".to_owned(),
            showCompleted: true,
        }
    }
}

/// The events `app.filterpanel` reports.
#[derive(Debug, Clone)]
pub enum FilterpanelEvent {
}

impl FilterpanelEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const PROJECTSWITCH: &str = "app.filterpanel.projectSwitch";
    pub const LISTTABERRANDS: &str = "app.filterpanel.listTabErrands";
    pub const LISTTABREADING: &str = "app.filterpanel.listTabReading";
    pub const LISTTABGARDEN: &str = "app.filterpanel.listTabGarden";
    pub const NEWLIST: &str = "app.filterpanel.newList";
    pub const SPLITVIEW: &str = "app.filterpanel.splitView";
    pub const HIDEPREVIEW: &str = "app.filterpanel.hidePreview";
    pub const SETTINGS: &str = "app.filterpanel.settings";
    pub const TRASHBUTTON: &str = "app.filterpanel.trashButton";
    pub const PALETTEBUTTON: &str = "app.filterpanel.paletteButton";
    pub const ACCOUNTBUTTON: &str = "app.filterpanel.accountButton";
    pub const SEARCHFIELD: &str = "app.filterpanel.searchField";
    pub const SEARCHOPTIONS: &str = "app.filterpanel.searchOptions";
    pub const SEARCHCOUNT: &str = "app.filterpanel.searchCount";
    pub const SEARCHCLEAR: &str = "app.filterpanel.searchClear";
    pub const NEWTASK: &str = "app.filterpanel.newTask";
    pub const SCOPEWHOLEPROJECT: &str = "app.filterpanel.scopeWholeProject";
    pub const FIELDSNOTESONLY: &str = "app.filterpanel.fieldsNotesOnly";
    pub const ROW: &str = "app.filterpanel.row";
    pub const ROWPICKUP: &str = "app.filterpanel.rowPickUp";
    pub const ROWBIKE: &str = "app.filterpanel.rowBike";
    pub const ROWGROCERIES: &str = "app.filterpanel.rowGroceries";
    pub const ROWSOURDOUGH: &str = "app.filterpanel.rowSourdough";
    pub const ROWOATMILK: &str = "app.filterpanel.rowOatMilk";
    pub const ROWPOST: &str = "app.filterpanel.rowPost";
    pub const ROWRETURNLIBRARY: &str = "app.filterpanel.rowReturnLibrary";
    pub const ROWMORE: &str = "app.filterpanel.rowMore";
    pub const SUBTASKTOGGLE: &str = "app.filterpanel.subtaskToggle";
    pub const SUBTASKCOUNT: &str = "app.filterpanel.subtaskCount";
    pub const DONECHECK: &str = "app.filterpanel.doneCheck";
    pub const EMPTYSTATE: &str = "app.filterpanel.emptyState";
    pub const NOMATCHHINT: &str = "app.filterpanel.noMatchHint";
    pub const SORTANDFILTER: &str = "app.filterpanel.sortAndFilter";
    pub const DONECOUNT: &str = "app.filterpanel.doneCount";
    pub const DONEPERCENT: &str = "app.filterpanel.donePercent";
    pub const PROGRESSBAR: &str = "app.filterpanel.progressBar";
    pub const GROUPHEADINGERRANDS: &str = "app.filterpanel.groupHeadingErrands";
    pub const GROUPHEADINGREADING: &str = "app.filterpanel.groupHeadingReading";
    pub const PREVIEWEMPTY: &str = "app.filterpanel.previewEmpty";
    pub const PANELTITLE: &str = "app.filterpanel.panelTitle";
    pub const PANELSUBTITLE: &str = "app.filterpanel.panelSubtitle";
    pub const CLOSEBUTTON: &str = "app.filterpanel.closeButton";
    pub const SECTIONSORT: &str = "app.filterpanel.sectionSort";
    pub const SECTIONSHOW: &str = "app.filterpanel.sectionShow";
    pub const SECTIONREFINE: &str = "app.filterpanel.sectionRefine";
    pub const SECTIONDATE: &str = "app.filterpanel.sectionDate";
    pub const SUMMARY: &str = "app.filterpanel.summary";
    pub const SORTMENU: &str = "app.filterpanel.sortMenu";
    pub const SORTMANUAL: &str = "app.filterpanel.sortManual";
    pub const SORTCREATED: &str = "app.filterpanel.sortCreated";
    pub const SORTUPDATED: &str = "app.filterpanel.sortUpdated";
    pub const SORTALPHABETICAL: &str = "app.filterpanel.sortAlphabetical";
    pub const SORTDONELAST: &str = "app.filterpanel.sortDoneLast";
    pub const SORTASCENDING: &str = "app.filterpanel.sortAscending";
    pub const SORTDESCENDING: &str = "app.filterpanel.sortDescending";
    pub const CLEARSORT: &str = "app.filterpanel.clearSort";
    pub const SHOWALL: &str = "app.filterpanel.showAll";
    pub const SHOWACTIVE: &str = "app.filterpanel.showActive";
    pub const SHOWDONE: &str = "app.filterpanel.showDone";
    pub const CLEARSHOW: &str = "app.filterpanel.clearShow";
    pub const NOTESANY: &str = "app.filterpanel.notesAny";
    pub const NOTESWITH: &str = "app.filterpanel.notesWith";
    pub const NOTESWITHOUT: &str = "app.filterpanel.notesWithout";
    pub const ATTACHMENTSANY: &str = "app.filterpanel.attachmentsAny";
    pub const ATTACHMENTSWITH: &str = "app.filterpanel.attachmentsWith";
    pub const ATTACHMENTSWITHOUT: &str = "app.filterpanel.attachmentsWithout";
    pub const SUBTASKSANY: &str = "app.filterpanel.subtasksAny";
    pub const SUBTASKSWITH: &str = "app.filterpanel.subtasksWith";
    pub const SUBTASKSWITHOUT: &str = "app.filterpanel.subtasksWithout";
    pub const CLEARREFINE: &str = "app.filterpanel.clearRefine";
    pub const DATECREATED: &str = "app.filterpanel.dateCreated";
    pub const DATEUPDATED: &str = "app.filterpanel.dateUpdated";
    pub const DATERANGEBUTTON: &str = "app.filterpanel.dateRangeButton";
    pub const RANGEANYTIME: &str = "app.filterpanel.rangeAnyTime";
    pub const RANGELAST24HOURS: &str = "app.filterpanel.rangeLast24Hours";
    pub const RANGELAST7DAYS: &str = "app.filterpanel.rangeLast7Days";
    pub const RANGELAST30DAYS: &str = "app.filterpanel.rangeLast30Days";
    pub const RANGECUSTOM: &str = "app.filterpanel.rangeCustom";
    pub const CLEARDATE: &str = "app.filterpanel.clearDate";
    pub const DAYSCAPTION: &str = "app.filterpanel.daysCaption";
    pub const DAYSFEWER: &str = "app.filterpanel.daysFewer";
    pub const DAYSINPUT: &str = "app.filterpanel.daysInput";
    pub const DAYSMORE: &str = "app.filterpanel.daysMore";
    pub const DAYSUNIT: &str = "app.filterpanel.daysUnit";
    pub const RESETBUTTON: &str = "app.filterpanel.resetButton";
    pub const DONEBUTTON: &str = "app.filterpanel.doneButton";
}
