//! Generated from omnicui/contracts/settingsgeneral.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.settingsgeneral`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SettingsgeneralProps {
    pub dataset: String,
}

impl Default for SettingsgeneralProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
        }
    }
}

/// The events `app.settingsgeneral` reports.
#[derive(Debug, Clone)]
pub enum SettingsgeneralEvent {
}

impl SettingsgeneralEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const HEADING: &str = "app.settingsgeneral.heading";
    pub const DISCARDALL: &str = "app.settingsgeneral.discardAll";
    pub const SECTIONNAV: &str = "app.settingsgeneral.sectionNav";
    pub const GENERALTAB: &str = "app.settingsgeneral.generalTab";
    pub const SHORTCUTSTAB: &str = "app.settingsgeneral.shortcutsTab";
    pub const ABOUTTAB: &str = "app.settingsgeneral.aboutTab";
    pub const SEARCH: &str = "app.settingsgeneral.search";
    pub const CLEARSEARCH: &str = "app.settingsgeneral.clearSearch";
    pub const GENERALMATCHES: &str = "app.settingsgeneral.generalMatches";
    pub const CONTENTPANEL: &str = "app.settingsgeneral.contentPanel";
    pub const APPEARANCEGROUP: &str = "app.settingsgeneral.appearanceGroup";
    pub const APPEARANCEHEADER: &str = "app.settingsgeneral.appearanceHeader";
    pub const APPEARANCEDISCARD: &str = "app.settingsgeneral.appearanceDiscard";
    pub const TASKSGROUP: &str = "app.settingsgeneral.tasksGroup";
    pub const TASKSHEADER: &str = "app.settingsgeneral.tasksHeader";
    pub const TASKSDISCARD: &str = "app.settingsgeneral.tasksDiscard";
    pub const THEMETITLE: &str = "app.settingsgeneral.themeTitle";
    pub const THEMEDESCRIPTION: &str = "app.settingsgeneral.themeDescription";
    pub const THEMESYSTEM: &str = "app.settingsgeneral.themeSystem";
    pub const THEMELIGHT: &str = "app.settingsgeneral.themeLight";
    pub const THEMEDARK: &str = "app.settingsgeneral.themeDark";
    pub const REDUCEDMOTIONTITLE: &str = "app.settingsgeneral.reducedMotionTitle";
    pub const REDUCEDMOTIONDESCRIPTION: &str = "app.settingsgeneral.reducedMotionDescription";
    pub const REDUCEDMOTIONSYSTEM: &str = "app.settingsgeneral.reducedMotionSystem";
    pub const REDUCEDMOTIONON: &str = "app.settingsgeneral.reducedMotionOn";
    pub const REDUCEDMOTIONOFF: &str = "app.settingsgeneral.reducedMotionOff";
    pub const SHOWCOMPLETEDTITLE: &str = "app.settingsgeneral.showCompletedTitle";
    pub const SHOWCOMPLETEDDESCRIPTION: &str = "app.settingsgeneral.showCompletedDescription";
    pub const SHOWCOMPLETEDSWITCH: &str = "app.settingsgeneral.showCompletedSwitch";
    pub const EMPTYTITLE: &str = "app.settingsgeneral.emptyTitle";
    pub const EMPTYSUBTITLE: &str = "app.settingsgeneral.emptySubtitle";
}
