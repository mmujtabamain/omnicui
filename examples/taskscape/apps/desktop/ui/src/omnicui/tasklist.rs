//! Generated from omnicui/contracts/tasklist.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.tasklist`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TasklistProps {
    pub dataset: String,
    pub project: String,
    pub list: String,
    pub showCompleted: bool,
    pub split: bool,
}

impl Default for TasklistProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            project: "Home".to_owned(),
            list: "Errands".to_owned(),
            showCompleted: true,
            split: false,
        }
    }
}

/// The events `app.tasklist` reports.
#[derive(Debug, Clone)]
pub enum TasklistEvent {
    TaskPreviewed { title: String },
    ListOpened { list: String },
    Copied { count: i64 },
    DraftStarted { list: String },
    SplitClosed { list: String },
}

impl TasklistEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::TaskPreviewed { title } => ("taskPreviewed", serde_json::json!({ "title": title })),
            Self::ListOpened { list } => ("listOpened", serde_json::json!({ "list": list })),
            Self::Copied { count } => ("copied", serde_json::json!({ "count": count })),
            Self::DraftStarted { list } => ("draftStarted", serde_json::json!({ "list": list })),
            Self::SplitClosed { list } => ("splitClosed", serde_json::json!({ "list": list })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const SEARCH: &str = "app.tasklist.search";
    pub const SEARCHOPTIONS: &str = "app.tasklist.searchOptions";
    pub const CLEARSEARCH: &str = "app.tasklist.clearSearch";
    pub const MATCHCOUNT: &str = "app.tasklist.matchCount";
    pub const NEWTASK: &str = "app.tasklist.newTask";
    pub const CLOSESPLIT: &str = "app.tasklist.closeSplit";
    pub const SORTFILTER: &str = "app.tasklist.sortFilter";
    pub const SEARCHMENU: &str = "app.tasklist.searchMenu";
    pub const SCOPETHISLIST: &str = "app.tasklist.scopeThisList";
    pub const SCOPEWHOLEPROJECT: &str = "app.tasklist.scopeWholeProject";
    pub const SCOPEALLPROJECTS: &str = "app.tasklist.scopeAllProjects";
    pub const FIELDSTITLEANDNOTES: &str = "app.tasklist.fieldsTitleAndNotes";
    pub const FIELDSTITLEONLY: &str = "app.tasklist.fieldsTitleOnly";
    pub const FIELDSNOTESONLY: &str = "app.tasklist.fieldsNotesOnly";
    pub const GROUPHEADING: &str = "app.tasklist.groupHeading";
    pub const GROUPHEADINGREADING: &str = "app.tasklist.groupHeadingReading";
    pub const ROWRETURNBOOKS: &str = "app.tasklist.rowReturnBooks";
    pub const MATCHTRIMHEDGE: &str = "app.tasklist.matchTrimHedge";
    pub const SCROLLAREA: &str = "app.tasklist.scrollArea";
    pub const EMPTYSTATE: &str = "app.tasklist.emptyState";
    pub const ROW: &str = "app.tasklist.row";
    pub const CHECK: &str = "app.tasklist.check";
    pub const MORE: &str = "app.tasklist.more";
    pub const ROWGROCERIES: &str = "app.tasklist.rowGroceries";
    pub const ROWOATMILK: &str = "app.tasklist.rowOatMilk";
    pub const ROWPOSTPARCEL: &str = "app.tasklist.rowPostParcel";
    pub const ROWBOOKBIKE: &str = "app.tasklist.rowBookBike";
    pub const ROWNEWSUBTASK: &str = "app.tasklist.rowNewSubtask";
    pub const ROWLASTERRAND: &str = "app.tasklist.rowLastErrand";
    pub const DISCLOSURE: &str = "app.tasklist.disclosure";
    pub const SUBTASK: &str = "app.tasklist.subtask";
    pub const BADGEGROCERIES: &str = "app.tasklist.badgeGroceries";
    pub const BADGEBIKE: &str = "app.tasklist.badgeBike";
    pub const ATTACHMENTBADGE: &str = "app.tasklist.attachmentBadge";
    pub const BADGEHERO: &str = "app.tasklist.badgeHero";
    pub const MATCHMARK: &str = "app.tasklist.matchMark";
    pub const CHECKBIKE: &str = "app.tasklist.checkBike";
    pub const CHECKPOST: &str = "app.tasklist.checkPost";
    pub const CHECKCUT: &str = "app.tasklist.checkCut";
    pub const CHECKGROCERIES: &str = "app.tasklist.checkGroceries";
    pub const CHECKSOURDOUGH: &str = "app.tasklist.checkSourdough";
    pub const MORESOURDOUGH: &str = "app.tasklist.moreSourdough";
    pub const ROWMENU: &str = "app.tasklist.rowMenu";
    pub const MENUCOPY: &str = "app.tasklist.menuCopy";
    pub const MENUADDSUBTASK: &str = "app.tasklist.menuAddSubtask";
    pub const MENURENAME: &str = "app.tasklist.menuRename";
    pub const MENUMOVETOLIST: &str = "app.tasklist.menuMoveToList";
    pub const MOVETARGETREADING: &str = "app.tasklist.moveTargetReading";
    pub const MENUPROMOTE: &str = "app.tasklist.menuPromote";
    pub const MENUDELETE: &str = "app.tasklist.menuDelete";
    pub const RENAMEFIELD: &str = "app.tasklist.renameField";
    pub const SUBTASKCOMPOSER: &str = "app.tasklist.subtaskComposer";
    pub const DONECOUNT: &str = "app.tasklist.doneCount";
    pub const PERCENT: &str = "app.tasklist.percent";
    pub const SELECTIONCOUNT: &str = "app.tasklist.selectionCount";
    pub const BULKBAR: &str = "app.tasklist.bulkBar";
    pub const BULKDONE: &str = "app.tasklist.bulkDone";
    pub const BULKUNDONE: &str = "app.tasklist.bulkUndone";
    pub const BULKMOVE: &str = "app.tasklist.bulkMove";
    pub const BULKCOPY: &str = "app.tasklist.bulkCopy";
    pub const BULKDELETE: &str = "app.tasklist.bulkDelete";
    pub const BULKCLEAR: &str = "app.tasklist.bulkClear";
}
