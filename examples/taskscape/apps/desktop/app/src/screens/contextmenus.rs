//! contract: omnicui/contracts/contextmenus.contract.yaml (id: app.contextmenus). The context menus of the main window (the project menu, a list tab's menu, a task row's menu with its Move to list submenu) and the sheets and toast they lead to, tested in the whole main window.
use crate::screens::appbar::{PROJECT_RENAME_SPRING, PROJECT_ROW_LAST, TAB_DELETE_READING};
use crate::screens::mainwindow::MainWindow;
use crate::screens::tasklist::{MOVE_SUBMENU, MOVE_TARGET_GARDEN};
use crate::App;
use taskscape_ui::omnicui::contextmenus::ContextmenusProps;
use taskscape_ui::{as_part, Look, Mounted};

/// The names a part carries in the main window (its screen's, or the window's own), under the context
/// menus' names (the web's contextmenu-parts.ts SOURCES).
const RENAMES: &[(&str, &str)] = &[
    ("app.mainwindow.projectSwitcher", "app.contextmenus.projectSwitch"),
    ("app.appbar.projectMenu", "app.contextmenus.projectMenu"),
    ("app.mainwindow.projectMenuHome", "app.contextmenus.projectHome"),
    ("app.mainwindow.projectMenuSpring", "app.contextmenus.projectSpringLaunch"),
    (PROJECT_ROW_LAST, "app.contextmenus.projectLast"),
    ("app.mainwindow.projectNew", "app.contextmenus.projectNew"),
    (PROJECT_RENAME_SPRING, "app.contextmenus.projectRenameSpring"),
    ("app.appbar.projectDeleteSpring", "app.contextmenus.projectDeleteSpring"),
    ("app.mainwindow.listTab", "app.contextmenus.firstTab"),
    ("app.mainwindow.listTabReading", "app.contextmenus.tabReading"),
    ("app.mainwindow.listTabGarden", "app.contextmenus.tabGarden"),
    ("app.mainwindow.deleteListReading", "app.contextmenus.tabCloseReading"),
    (TAB_DELETE_READING, "app.contextmenus.tabCloseReading"),
    ("app.appbar.tabMenu", "app.contextmenus.tabMenu"),
    ("app.appbar.tabMenuRename", "app.contextmenus.menuTabRename"),
    ("app.appbar.tabMenuSplit", "app.contextmenus.menuTabSplit"),
    ("app.appbar.tabMenuExport", "app.contextmenus.menuTabExport"),
    ("app.appbar.tabMenuDelete", "app.contextmenus.menuTabDelete"),
    ("app.mainwindow.rowMenu", "app.contextmenus.rowMenu"),
    ("app.tasklist.menuCopy", "app.contextmenus.menuCopy"),
    ("app.tasklist.menuAddSubtask", "app.contextmenus.menuAddSubtask"),
    ("app.tasklist.menuRename", "app.contextmenus.menuRename"),
    ("app.tasklist.menuMoveToList", "app.contextmenus.menuMoveToList"),
    ("app.tasklist.menuDelete", "app.contextmenus.menuDelete"),
    ("app.tasklist.moveTargetReading", "app.contextmenus.moveReading"),
    (MOVE_SUBMENU, "app.contextmenus.moveSubmenu"),
    (MOVE_TARGET_GARDEN, "app.contextmenus.moveGarden"),
    ("app.mainwindow.rowTitleEdit", "app.contextmenus.titleInput"),
    ("app.tasklist.subtaskComposer", "app.contextmenus.subtaskInput"),
    ("app.mainwindow.dialogHeading", "app.contextmenus.sheetHeadline"),
    ("app.mainwindow.dialogDetail", "app.contextmenus.sheetDetail"),
    ("app.mainwindow.dialogName", "app.contextmenus.sheetInput"),
    ("app.mainwindow.dialogCreate", "app.contextmenus.sheetAccept"),
    ("app.mainwindow.dialogDelete", "app.contextmenus.sheetAccept"),
    ("app.mainwindow.dialogMoveToTrash", "app.contextmenus.sheetAccept"),
    ("app.mainwindow.dialogAccept", "app.contextmenus.sheetAccept"),
    ("app.mainwindow.dialogCancel", "app.contextmenus.sheetCancel"),
    ("app.toasthost.message", "app.contextmenus.toast"),
    ("app.mainwindow.toastUndo", "app.contextmenus.toastUndo"),
    ("app.toasthost.dismiss", "app.contextmenus.toastDismiss"),
    // Task rows, named by their title (the window names some of them already).
    ("app.mainwindow.rowBike", "app.contextmenus.rowBikeService"),
];

/// Rows the context menus' tests name that the main window does not: their title buttons and More buttons.
const TITLE_CLAIMS: &[(&str, &str)] = &[("Pick up prescription", "app.contextmenus.rowPrescription"), ("Buy eggs", "app.contextmenus.rowEggs"), ("Renew passport", "app.contextmenus.subtaskTitle")];
const MORE_CLAIMS: &[(&str, &str)] = &[
    ("Pick up prescription", "app.contextmenus.morePrescription"),
    ("Book bike service", "app.contextmenus.moreBikeService"),
    ("Cut a spare key", "app.contextmenus.moreCutKey"),
    ("Buy groceries for the week", "app.contextmenus.moreGroceries"),
];

pub struct ContextMenus {
    window: MainWindow,
}

impl Default for ContextMenus {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMenus {
    pub fn new() -> Self {
        let mut window = MainWindow::new();
        let left = window.left_pane();
        left.claims.extend_from_slice(TITLE_CLAIMS);
        left.more_claims = MORE_CLAIMS.to_vec();
        Self { window }
    }

    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) {
        as_part(ui, RENAMES, |ui| {
            self.window.show(app, ui, look);
        });
    }
}

struct Mount {
    app: App,
    screen: ContextMenus,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, _emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        self.screen.show(&mut self.app, ui, look);
    }
}

pub fn mount(app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let _p: ContextmenusProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    Ok(Box::new(Mount { app, screen: ContextMenus::new() }))
}
