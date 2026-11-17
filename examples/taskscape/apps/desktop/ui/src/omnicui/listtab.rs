//! Generated from omnicui/contracts/listtab.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.listtab`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ListtabProps {
    pub name: String,
    pub selected: bool,
    pub editing: bool,
    pub dropTarget: bool,
    pub dragging: bool,
    pub paneMarker: bool,
}

impl Default for ListtabProps {
    fn default() -> Self {
        Self {
            name: "".to_owned(),
            selected: false,
            editing: false,
            dropTarget: false,
            dragging: false,
            paneMarker: false,
        }
    }
}

/// The events `ui.listtab` reports.
#[derive(Debug, Clone)]
pub enum ListtabEvent {
    Select {  },
    RenameStart {  },
    RenameCommit { name: String },
    RenameCancel {  },
    Delete {  },
    ContextMenu {  },
    DropTask { taskId: String },
    Reorder { fromIndex: i64, toIndex: i64 },
}

impl ListtabEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Select {  } => ("select", serde_json::json!({  })),
            Self::RenameStart {  } => ("renameStart", serde_json::json!({  })),
            Self::RenameCommit { name } => ("renameCommit", serde_json::json!({ "name": name })),
            Self::RenameCancel {  } => ("renameCancel", serde_json::json!({  })),
            Self::Delete {  } => ("delete", serde_json::json!({  })),
            Self::ContextMenu {  } => ("contextMenu", serde_json::json!({  })),
            Self::DropTask { taskId } => ("dropTask", serde_json::json!({ "taskId": taskId })),
            Self::Reorder { fromIndex, toIndex } => ("reorder", serde_json::json!({ "fromIndex": fromIndex, "toIndex": toIndex })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const WRAPPER: &str = "ui.listtab.wrapper";
    pub const TAB: &str = "ui.listtab.tab";
    pub const DELETEBUTTON: &str = "ui.listtab.deleteButton";
    pub const RENAMEINPUT: &str = "ui.listtab.renameInput";
    pub const PANEMARKER: &str = "ui.listtab.paneMarker";
}
