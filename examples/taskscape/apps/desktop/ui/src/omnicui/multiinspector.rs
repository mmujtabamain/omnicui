//! Generated from omnicui/contracts/multiinspector.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.multiinspector`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct MultiinspectorProps {
    pub dataset: String,
    pub list: String,
    pub selection: String,
}

impl Default for MultiinspectorProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            list: "Errands".to_owned(),
            selection: "four".to_owned(),
        }
    }
}

/// The events `app.multiinspector` reports.
#[derive(Debug, Clone)]
pub enum MultiinspectorEvent {
}

impl MultiinspectorEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const COUNT: &str = "app.multiinspector.count";
    pub const HEADLINE: &str = "app.multiinspector.headline";
    pub const TALLY: &str = "app.multiinspector.tally";
    pub const CLEARSELECTION: &str = "app.multiinspector.clearSelection";
    pub const SECTIONLABEL: &str = "app.multiinspector.sectionLabel";
    pub const SELECTEDLIST: &str = "app.multiinspector.selectedList";
    pub const ROW: &str = "app.multiinspector.row";
    pub const ROWCHECK: &str = "app.multiinspector.rowCheck";
    pub const ROWTITLE: &str = "app.multiinspector.rowTitle";
    pub const ROWLISTBADGE: &str = "app.multiinspector.rowListBadge";
    pub const ROWOPEN: &str = "app.multiinspector.rowOpen";
    pub const PARCELCHECK: &str = "app.multiinspector.parcelCheck";
    pub const PARCELTITLE: &str = "app.multiinspector.parcelTitle";
    pub const LASTROWTITLE: &str = "app.multiinspector.lastRowTitle";
    pub const INSPECTOR: &str = "app.multiinspector.inspector";
}
