//! contract: omnicui/contracts/mainwindow.contract.yaml (id: app.mainwindow). The main window: the title bar, the task pane (two in split view, with the divider between), the aside (the task's inspector, the draft, the multi-select inspector or Filter & Sort, with its divider) and the toast, composed from the screens the contracts check one at a time, plus the palette panel and the windows it opens.
//!
//! The other windows (Trash, Settings, a confirmation or prompt) are real OS windows in the app
//! (src/main.rs opens them as eframe viewports). A screen test has one window, so the test mount draws
//! each open window as a panel over this one, with the same content.
use crate::screens::{
    appbar::AppBar, draftinspector::DraftInspector, filterpanel::FilterPanel, multiinspector::MultiInspector, palette::Palette, settingswindow::SettingsWindow, sheets::Sheets, taskinspector::TaskInspector,
    tasklist::TaskList, toasthost::ToastHost, trashwindow::TrashWindow,
};
use crate::state::{Aside, PREVIEW_MAX, PREVIEW_MIN, SPLIT_MAX, SPLIT_MIN};
use crate::App;
use egui::{Align2, CornerRadius, FontId, Id, Key, Pos2, Rect, Role, Sense, Stroke, StrokeKind, Ui, UiBuilder, WidgetInfo};
use taskscape_ui::omnicui::mainwindow::{parts, MainwindowEvent, MainwindowProps};
use taskscape_ui::omnicui::resizer::{parts::RESIZER as RESIZER_HANDLE, ResizerProps};
use taskscape_ui::widgets::resizer::Resizer;
use taskscape_ui::{as_part, part, rem, weight, Look, Mounted};

/// The title bar's height (the web's h-[3.286rem]).
const BAR: f32 = 46.0;

/// The task rows the main window's tests name by their text (the web's TITLES), on the left pane.
const CLAIMS: &[(&str, &str)] = &[
    ("Book bike service", "app.mainwindow.rowBike"),
    ("Cut a spare key", "app.mainwindow.rowKeysTitle"),
    ("Buy groceries for the week", "app.mainwindow.rowGroceries"),
    ("Rewrite the hero section", "app.mainwindow.rowHeroRewrite"),
    ("Oat milk", "app.mainwindow.subtaskOatMilk"),
    ("soil health", "app.mainwindow.matchRow"),
];

/// The child screens' parts, under the main window's names (the web's main-window/parts.ts).
const RENAMES: &[(&str, &str)] = &[
    ("app.appbar.projectSwitcher", "app.mainwindow.projectSwitcher"),
    ("app.appbar.projectNew", "app.mainwindow.projectNew"),
    ("app.appbar.projectRow", "app.mainwindow.projectMenuHome"),
    ("app.appbar.projectRowSpring", "app.mainwindow.projectMenuSpring"),
    ("app.appbar.listTabs", "app.mainwindow.listTabs"),
    ("app.appbar.listTab", "app.mainwindow.listTab"),
    ("app.appbar.listTabReading", "app.mainwindow.listTabReading"),
    ("app.appbar.listTabGarden", "app.mainwindow.listTabGarden"),
    ("app.appbar.listTabDelete", "app.mainwindow.deleteList"),
    (crate::screens::appbar::TAB_DELETE_READING, "app.mainwindow.deleteListReading"),
    (crate::screens::appbar::TAB_HARDWARE, "app.mainwindow.listTabNew"),
    (crate::screens::palette::ROW_OPEN_TRASH, "app.mainwindow.paletteOpenTrash"),
    ("app.appbar.newList", "app.mainwindow.newList"),
    ("app.appbar.splitView", "app.mainwindow.splitView"),
    ("app.appbar.swapPanes", "app.mainwindow.swapPanes"),
    ("app.appbar.previewToggle", "app.mainwindow.previewToggle"),
    ("app.appbar.trashButton", "app.mainwindow.trash"),
    ("app.appbar.settingsButton", "app.mainwindow.settingsButton"),
    ("app.tasklist.search", "app.mainwindow.searchField"),
    ("app.tasklist.searchOptions", "app.mainwindow.searchOptions"),
    ("app.tasklist.searchMenu", "app.mainwindow.scopeMenu"),
    ("app.tasklist.scopeWholeProject", "app.mainwindow.scopeWholeProject"),
    ("app.tasklist.newTask", "app.mainwindow.newTask"),
    ("app.tasklist.groupHeading", "app.mainwindow.groupHeading"),
    ("app.tasklist.sortFilter", "app.mainwindow.sortFilter"),
    ("app.tasklist.doneCount", "app.mainwindow.doneCount"),
    ("app.tasklist.percent", "app.mainwindow.footerPercent"),
    ("app.tasklist.selectionCount", "app.mainwindow.selectionCount"),
    ("app.tasklist.bulkDone", "app.mainwindow.bulkMarkDone"),
    ("app.tasklist.more", "app.mainwindow.rowMore"),
    ("app.tasklist.rowMenu", "app.mainwindow.rowMenu"),
    ("app.tasklist.renameField", "app.mainwindow.rowTitleEdit"),
    ("app.tasklist.badgeGroceries", "app.mainwindow.subtaskCount"),
    ("app.tasklist.disclosure", "app.mainwindow.rowGroceriesExpand"),
    ("app.tasklist.checkBike", "app.mainwindow.checkBike"),
    ("app.tasklist.checkCut", "app.mainwindow.rowKeysSelect"),
    ("app.tasklist.check", "app.mainwindow.rowSelect"),
    ("app.tasklist.row", "app.mainwindow.rowTitle"),
    ("app.tasklist.attachmentBadge", "app.mainwindow.attachmentBadge"),
    ("app.multiinspector.clearSelection", "app.mainwindow.clearSelection"),
    ("app.multiinspector.rowOpen", "app.mainwindow.openSelection"),
    ("app.multiinspector.inspector", "app.mainwindow.multiInspector"),
    ("app.taskinspector.title", "app.mainwindow.previewTitle"),
    ("app.trashwindow.title", "app.mainwindow.trashHeading"),
    ("app.filterpanel.panelTitle", "app.mainwindow.filterHeading"),
    ("app.sheets.card", "app.mainwindow.dialog"),
    ("app.sheets.headline", "app.mainwindow.dialogHeading"),
    ("app.sheets.nameField", "app.mainwindow.dialogName"),
    ("app.sheets.cancel", "app.mainwindow.dialogCancel"),
    ("app.sheets.detail", "app.mainwindow.dialogDetail"),
    ("app.toasthost.notice", "app.mainwindow.toast"),
    ("app.toasthost.undo", "app.mainwindow.toastUndo"),
    ("app.palette.panel", "app.mainwindow.commandPalette"),
    ("app.palette.search", "app.mainwindow.commandInput"),
    ("app.settingswindow.title", "app.mainwindow.settingsHeading"),
];

pub struct MainWindow {
    bar: AppBar,
    panes: [TaskList; 2],
    inspector: TaskInspector,
    draft: DraftInspector,
    multi: MultiInspector,
    filter: FilterPanel,
    toast: ToastHost,
    palette: Palette,
    sheets: Sheets,
    trash: TrashWindow,
    settings: SettingsWindow,
    split_divider: Resizer,
    preview_divider: Resizer,
    /// The other windows are drawn over this one (a screen test has one window). The app sets it off and
    /// opens them as OS windows with [`MainWindow::windows`].
    pub in_place: bool,
    /// Which of Trash, Settings and a sheet were open last frame, to focus a window when it opens.
    open_last: (bool, bool, bool),
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            bar: AppBar::new(),
            panes: {
                let mut left = TaskList::new(0);
                left.claims = CLAIMS.to_vec();
                left.row_box = Some(("Pick up prescription", "app.mainwindow.row"));
                let mut right = TaskList::new(1);
                right.row_box = Some(("", "app.mainwindow.splitRowBox"));
                [left, right]
            },
            inspector: TaskInspector::new(),
            draft: DraftInspector::new(),
            multi: MultiInspector::new(),
            filter: FilterPanel::new(),
            toast: ToastHost::new(),
            palette: Palette::new(),
            sheets: Sheets::new(),
            trash: TrashWindow::new(),
            settings: SettingsWindow::new(),
            split_divider: Resizer::new(ResizerProps { value: 0.5, min: SPLIT_MIN as f64, max: SPLIT_MAX as f64, reset: "double-click".into(), open: true, label: "Resize the panes".into(), ..Default::default() }),
            preview_divider: Resizer::new(ResizerProps { value: 322.0, min: PREVIEW_MIN as f64, max: PREVIEW_MAX as f64, reset: "none".into(), open: true, label: "Resize the preview panel".into(), ..Default::default() }),
            in_place: true,
            open_last: (false, false, false),
        }
    }

    /// One frame of the main window, filling `ui`. The windows it opens are drawn by [`MainWindow::windows`].
    pub fn show(&mut self, app: &mut App, ui: &mut Ui, look: Look) -> Vec<MainwindowEvent> {
        let t = look.t;
        let full = ui.max_rect();
        ui.painter().rect_filled(full, 0.0, t.surface_1);
        node(ui, full, "mainwindow.window", parts::WINDOW, "Taskscape");
        self.keys(app, ui);
        // While a pane's menu is open, a press anywhere else closes it and goes no further (the web's menu
        // overlay): a layer over the window, under the menus, takes it.
        if self.panes.iter().any(|p| p.menu_open()) {
            let ctx = ui.ctx().clone();
            let closed = egui::Area::new(Id::new("mainwindow.menublocker")).order(egui::Order::Middle).fixed_pos(full.min).show(&ctx, |ui| {
                let r = ui.interact(full, Id::new("mainwindow.menublocker.hit"), Sense::click());
                r.clicked() || r.secondary_clicked()
            });
            if closed.inner {
                for p in &mut self.panes {
                    p.close_menus();
                }
            }
        }

        as_part(ui, RENAMES, |ui| {
            // The title bar.
            let bar = Rect::from_min_size(full.min, egui::vec2(full.width(), BAR));
            node(ui, bar, "mainwindow.titlebar", parts::TITLEBAR, "Title bar");
            ui.scope_builder(UiBuilder::new().max_rect(bar), |ui| {
                self.bar.show(app, ui, look);
            });

            let body = Rect::from_min_max(Pos2::new(full.min.x, bar.max.y), full.max);
            self.carry(app, ui, body);
            let draft_before = app.ui.draft_list.is_some();
            let project_lists: Vec<String> = app.ui.project.as_deref().map(|p| app.data.lists_of(p).iter().map(|l| l.id.clone()).collect()).unwrap_or_default();
            let lists = project_lists.len();
            // The left pane shows the first list of the project unless its list is still there.
            if lists > 0 && !app.ui.panes[0].as_ref().is_some_and(|l| project_lists.contains(l)) {
                app.ui.panes[0] = Some(project_lists[0].clone());
            }
            // Split view: the right pane is the list after the left one, unless it holds another list.
            if app.ui.split_on() && lists >= 2 {
                let left = app.ui.panes[0].clone();
                let right = app.ui.panes[1].clone();
                if right.is_none() || right == left || !right.as_ref().is_some_and(|r| project_lists.contains(r)) {
                    let at = left.as_ref().and_then(|l| project_lists.iter().position(|x| x == l)).unwrap_or(0);
                    app.ui.panes[1] = Some(project_lists[(at + 1) % lists].clone());
                }
            }
            let aside_shown = app.ui.aside != Aside::None;
            let aside_w = if aside_shown { app.ui.preview_width.clamp(PREVIEW_MIN, PREVIEW_MAX) } else { 0.0 };
            let panes_rect = Rect::from_min_max(body.min, Pos2::new(body.max.x - aside_w, body.max.y));

            if lists == 0 {
                self.no_lists(ui, panes_rect, look);
            } else if app.ui.split_on() {
                let left_w = (panes_rect.width() * app.ui.split_ratio.clamp(SPLIT_MIN, SPLIT_MAX)).round();
                let left = Rect::from_min_size(panes_rect.min, egui::vec2(left_w, panes_rect.height()));
                let right = Rect::from_min_max(Pos2::new(left.max.x, panes_rect.min.y), panes_rect.max);
                self.pane(app, ui, left, 0, look);
                self.pane(app, ui, right, 1, look);
                // The divider sits over the edge between the panes, taking no width.
                let strip = Rect::from_center_size(Pos2::new(left.max.x, panes_rect.center().y), egui::vec2(6.0, panes_rect.height()));
                self.split_divider_ui(app, ui, strip, panes_rect, look);
            } else {
                self.pane(app, ui, panes_rect, 0, look);
            }

            // A draft the panes opened this frame (New task pressed with Enter) must not take that Enter too.
            if app.ui.draft_list.is_some() && !draft_before {
                ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
            }
            if aside_shown {
                let aside = Rect::from_min_max(Pos2::new(panes_rect.max.x, body.min.y), body.max);
                ui.painter().rect_filled(aside, 0.0, t.surface_1);
                ui.painter().vline(aside.min.x, aside.y_range(), Stroke::new(1.0, t.edge_1));
                self.aside(app, ui, aside, look);
                let strip = Rect::from_center_size(Pos2::new(aside.min.x, aside.center().y), egui::vec2(6.0, aside.height()));
                self.preview_divider_ui(app, ui, strip, look);
            }

            // Over everything: the toast, then the palette panel and the windows the test draws in place.
            ui.scope_builder(UiBuilder::new().max_rect(full), |ui| {
                self.toast.show(app, ui, look);
            });
            if app.ui.palette_open {
                ui.scope_builder(UiBuilder::new().max_rect(full), |ui| {
                    self.palette.show(app, ui, look);
                });
            }
            if self.in_place {
                self.windows_in_place(app, ui, full, look);
            }
        });
        vec![]
    }

    /// The windows the main window opens, drawn as panels over it (a test has one window; the app opens
    /// them as OS windows with the same content, see src/main.rs).
    fn windows_in_place(&mut self, app: &mut App, ui: &mut Ui, full: Rect, look: Look) {
        let t = look.t;
        let mut panel = |ui: &mut Ui, size: egui::Vec2, id: &str, part_id: &str, label: &str, f: &mut dyn FnMut(&mut Ui)| {
            let r = Rect::from_center_size(full.center(), size.min(full.size() - egui::vec2(16.0, 16.0)));
            egui::Area::new(Id::new(id)).order(egui::Order::Foreground).fixed_pos(r.min).show(ui.ctx(), |ui| {
                ui.set_clip_rect(r);
                ui.painter().rect(r, CornerRadius::same(10), t.surface_1, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
                node(ui, r, id, part_id, label);
                ui.scope_builder(UiBuilder::new().max_rect(r), |ui| f(ui));
            });
        };
        if app.trash_window {
            let trash = &mut self.trash;
            let titles = app.data.trash.iter().map(|t| t.title.as_str()).collect::<Vec<_>>().join(", ");
            panel(ui, egui::vec2(520.0, 560.0), "mainwindow.trashwin", parts::TRASHPANE, &titles, &mut |ui| {
                trash.show(app_ptr(app), ui, look);
            });
        }
        if app.settings_window.is_some() {
            let settings = &mut self.settings;
            panel(ui, egui::vec2(640.0, 520.0), "mainwindow.settingswin", parts::SETTINGSDIALOG, "Settings", &mut |ui| {
                settings.show(app_ptr(app), ui, look);
            });
        }
        if let Some(sheet) = app.sheet.clone() {
            let sheets = &mut self.sheets;
            let accept = accept_part(&sheet);
            panel(ui, egui::vec2(380.0, 230.0), "mainwindow.sheetwin", "app.mainwindow.sheetWindow", "", &mut |ui| {
                as_part(ui, &[("app.sheets.accept", accept)], |ui| sheets.show(app_ptr(app), ui, look));
            });
        }
    }

    /// The Trash, Settings and a confirmation, each as an OS window of its own (eframe viewports). Closing
    /// one with the OS's close button closes it here too.
    pub fn windows(&mut self, app: &mut App, ctx: &egui::Context, look: Look) {
        use crate::chrome;
        let open_before = (app.trash_window, app.settings_window.is_some(), app.sheet.is_some());
        // A window that has just opened takes the keyboard and comes to the front (the reference's set_focus).
        let opened = (open_before.0 && !self.open_last.0, open_before.1 && !self.open_last.1, open_before.2 && !self.open_last.2);
        let t = look.t;
        let frame = egui::Frame::NONE.fill(t.surface_1);
        let native = app.native_window;
        let header = |ui: &egui::Ui| egui::Rect::from_min_size(ui.max_rect().min, egui::vec2(ui.max_rect().width(), 46.0));
        // Each window opens centred over the main window.
        let main = ctx.input(|i| i.viewport().outer_rect);
        let at = |w: f32, h: f32| main.map(|r| r.center() - egui::vec2(w, h) / 2.0).unwrap_or(egui::pos2(80.0, 80.0));
        if app.trash_window {
            let vp = chrome::panel(egui::ViewportBuilder::default().with_title("Trash").with_inner_size([520.0, 560.0]).with_position(at(520.0, 560.0)).with_min_inner_size([420.0, 360.0]).with_active(true));
            let id = egui::ViewportId::from_hash_of("trash");
            if opened.0 {
                ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Focus);
            }
            ctx.show_viewport_immediate(id, vp, |ctx, _| {
                egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
                    let bar = header(ui);
                    chrome::drag_region(ui, bar, false);
                    self.trash.lead = chrome::lead(native);
                    self.trash.show(app, ui, look);
                    let trail = chrome::trail(native, 1);
                    if trail > 0.0 && chrome::buttons(ui, look, egui::Rect::from_min_max(egui::pos2(bar.right() - trail, bar.top()), bar.max), false) {
                        app.trash_window = false;
                    }
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    app.trash_window = false;
                }
            });
        }
        if app.settings_window.is_some() {
            let vp = chrome::panel(egui::ViewportBuilder::default().with_title("Settings").with_inner_size([640.0, 520.0]).with_position(at(640.0, 520.0)).with_min_inner_size([640.0, 520.0]).with_active(true));
            let id = egui::ViewportId::from_hash_of("settings");
            if opened.1 {
                ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Focus);
            }
            ctx.show_viewport_immediate(id, vp, |ctx, _| {
                egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
                    let bar = header(ui);
                    chrome::drag_region(ui, bar, false);
                    let trail = chrome::trail(native, 1);
                    self.settings.general_chrome = (chrome::lead(native), trail);
                    self.settings.show(app, ui, look);
                    if trail > 0.0 && chrome::buttons(ui, look, egui::Rect::from_min_max(egui::pos2(bar.right() - trail, bar.top()), bar.max), false) {
                        app.settings_window = None;
                    }
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    app.settings_window = None;
                }
            });
        }
        if app.sheet.is_some() {
            let vp = chrome::sheet(egui::ViewportBuilder::default().with_title("Taskscape").with_inner_size([380.0, 140.0]).with_position(at(380.0, 140.0)).with_active(true));
            let id = egui::ViewportId::from_hash_of("sheet");
            if opened.2 {
                ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Focus);
            }
            ctx.show_viewport_immediate(id, vp, |ctx, _| {
                // The sheet's card is the whole window: no fill around it (the window is transparent).
                egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| {
                    self.sheets.show(app, ui, look);
                });
                // The window fits the card: its height follows the card's.
                let want = egui::vec2(380.0, (self.sheets.card_height() + 18.0).max(120.0));
                if self.sheets.card_height() > 0.0 && ctx.input(|i| i.viewport().inner_rect).is_some_and(|r| (r.height() - want.y).abs() > 1.0) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(want));
                }
                if ctx.input(|i| i.viewport().close_requested()) {
                    app.sheet = None;
                }
            });
        }
        self.open_last = open_before;
        // A window that closed itself from its own keys: the main window draws again, so the OS window goes.
        if open_before != (app.trash_window, app.settings_window.is_some(), app.sheet.is_some()) {
            ctx.request_repaint_of(egui::ViewportId::ROOT);
        }
    }

    /// Dragged tasks: dropped on a list tab they move to that list; carried onto the other pane they
    /// become that pane's drag, so its rows take the drop.
    fn carry(&mut self, app: &mut App, ui: &mut Ui, body: Rect) {
        let (pointer, released) = ui.input(|i| (i.pointer.interact_pos(), i.pointer.primary_released()));
        let Some(p) = pointer else { return };
        let from = (0..2).find(|&i| !self.panes[i].dragged().is_empty());
        let Some(from) = from else { return };
        if released {
            if let Some((list, _)) = self.bar.tab_rects.iter().find(|(_, r)| r.contains(p)).cloned() {
                let ids = self.panes[from].dragged().to_vec();
                self.panes[from].set_dragged(vec![]);
                for id in ids {
                    if app.data.task(&id).is_some_and(|t| t.list_id != list) {
                        crate::screens::tasklist::move_tree(app, &id, None, &list, None);
                    }
                }
                return;
            }
        }
        if app.ui.split_on() {
            let lists = app.ui.project.as_deref().map(|pr| app.data.lists_of(pr).len()).unwrap_or(0);
            let aside_w = if app.ui.aside != Aside::None { app.ui.preview_width.clamp(PREVIEW_MIN, PREVIEW_MAX) } else { 0.0 };
            let panes = Rect::from_min_max(body.min, Pos2::new(body.max.x - aside_w, body.max.y));
            let split_x = panes.min.x + (panes.width() * app.ui.split_ratio.clamp(SPLIT_MIN, SPLIT_MAX)).round();
            let over = if p.x < split_x { 0 } else { 1 };
            if lists > 0 && over != from && panes.contains(p) {
                let ids = self.panes[from].dragged().to_vec();
                self.panes[from].set_dragged(vec![]);
                self.panes[over].set_dragged(ids);
            }
        }
    }

    /// The left (or only) pane, for a window that names more of its rows.
    pub fn left_pane(&mut self) -> &mut TaskList {
        &mut self.panes[0]
    }

    /// A task pane: the task list on a list, in its own container part.
    fn pane(&mut self, app: &mut App, ui: &mut Ui, rect: Rect, pane: usize, look: Look) {
        let id = if pane == 0 { parts::TASKPANE } else { parts::SPLITPANE };
        node(ui, rect, if pane == 0 { "mainwindow.taskpane" } else { "mainwindow.splitpane" }, id, "");
        // The right pane's own parts carry the split names.
        let right: &[(&str, &str)] = &[
            ("app.tasklist.search", "app.mainwindow.splitSearch"),
            ("app.tasklist.closeSplit", "app.mainwindow.splitClose"),
            ("app.tasklist.row", "app.mainwindow.splitRow"),
            ("app.tasklist.doneCount", "app.mainwindow.splitFooterDoneCount"),
        ];
        let renames: &[(&str, &str)] = if pane == 1 { right } else { &[] };
        let list = &mut self.panes[pane];
        as_part(ui, renames, |ui| {
            ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                ui.set_clip_rect(rect);
                list.show(app, ui, look);
            });
        });
    }

    /// The aside: Filter & Sort, the draft, the multi-select inspector, the task's inspector, or the empty preview.
    fn aside(&mut self, app: &mut App, ui: &mut Ui, rect: Rect, look: Look) {
        let t = look.t;
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            ui.set_clip_rect(rect);
            if app.ui.aside == Aside::Filter {
                node(ui, rect, "mainwindow.filterpanel", parts::FILTERPANEL, "Filter & Sort");
                self.filter.panel(app, ui, look);
            } else if app.ui.draft_list.is_some() {
                node(ui, rect, "mainwindow.preview", parts::PREVIEWPANEL, "Preview");
                node(ui, rect, "mainwindow.draft", parts::DRAFTINSPECTOR, "New task");
                self.draft.show(app, ui, look);
            } else if app.ui.selection.len() > 1 {
                self.multi.show(app, ui, look);
            } else if app.ui.preview.as_ref().is_some_and(|p| app.data.task(p).is_some()) {
                node(ui, rect, "mainwindow.preview", parts::PREVIEWPANEL, "Preview");
                self.inspector.show(app, ui, look);
            } else {
                node(ui, rect, "mainwindow.preview", parts::PREVIEWPANEL, "Preview");
                // The empty preview: an icon over "No task selected" and its hint.
                let c = rect.center();
                ui.painter().text(c - egui::vec2(0.0, 26.0), Align2::CENTER_CENTER, taskscape_ui::icon("ads_click", 24.0).text(), FontId::new(24.0, egui::FontFamily::Name("Icons".into())), t.content_3);
                let title = ui.painter().text(c, Align2::CENTER_CENTER, "No task selected", FontId::new(rem(1.071), weight(500)), t.content_2);
                ui.painter().text(c + egui::vec2(0.0, 22.0), Align2::CENTER_CENTER, "Select a task to inspect it", FontId::new(rem(0.893), weight(400)), t.content_3);
                let r = ui.interact(title.expand2(egui::vec2(60.0, 20.0)), Id::new("mainwindow.previewempty"), Sense::hover());
                r.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "No task selected"));
                ui.ctx().accesskit_node_builder(r.id, |n| n.set_value("No task selected".to_owned()));
                part(ui, &r, parts::PREVIEWEMPTY);
            }
        });
    }

    /// No lists in the project: the pane says so, and split view is off.
    fn no_lists(&mut self, ui: &mut Ui, rect: Rect, look: Look) {
        let t = look.t;
        ui.painter().rect_filled(rect, 0.0, t.surface_0);
        let c = rect.center();
        let title = ui.painter().text(c - egui::vec2(0.0, 10.0), Align2::CENTER_CENTER, "No lists yet", FontId::new(rem(1.071), weight(500)), t.content_2);
        let sub = ui.painter().text(c + egui::vec2(0.0, 12.0), Align2::CENTER_CENTER, "Create a list to start adding tasks", FontId::new(rem(0.893), weight(400)), t.content_3);
        for (r, id, p, s) in [(title, "mainwindow.nolists.title", parts::NOLISTSTITLE, "No lists yet"), (sub, "mainwindow.nolists.sub", parts::NOLISTSSUBTITLE, "Create a list to start adding tasks")] {
            let resp = ui.interact(r, Id::new(id), Sense::hover());
            resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, s));
            ui.ctx().accesskit_node_builder(resp.id, |n| n.set_value(s.to_owned()));
            part(ui, &resp, p);
        }
    }

    fn split_divider_ui(&mut self, app: &mut App, ui: &mut Ui, strip: Rect, panes: Rect, look: Look) {
        self.split_divider.set_value(app.ui.split_ratio as f64);
        self.split_divider.set_span(panes.width());
        as_part(ui, &[(RESIZER_HANDLE, "app.mainwindow.splitDivider")], |ui| {
            ui.scope_builder(UiBuilder::new().max_rect(strip), |ui| self.split_divider.ui(ui, look))
        });
        app.ui.set_split_ratio(self.split_divider.value() as f32);
    }

    fn preview_divider_ui(&mut self, app: &mut App, ui: &mut Ui, strip: Rect, look: Look) {
        self.preview_divider.set_value(app.ui.preview_width as f64);
        as_part(ui, &[(RESIZER_HANDLE, "app.mainwindow.previewDivider")], |ui| {
            ui.scope_builder(UiBuilder::new().max_rect(strip), |ui| self.preview_divider.ui(ui, look))
        });
        app.ui.set_preview_width(self.preview_divider.value() as f32);
    }

    /// The window's own chords (the screens keep theirs: a list's search and New task, the title bar's
    /// list and project keys). Nothing global acts while a sheet or the palette is up.
    fn keys(&mut self, app: &mut App, ui: &mut Ui) {
        if app.sheet.is_some() || app.ui.palette_open {
            return;
        }
        let cmd = |i: &egui::InputState, shift: bool, key: Key| (i.modifiers.command || i.modifiers.ctrl) && i.modifiers.shift == shift && !i.modifiers.alt && i.key_pressed(key);
        let (palette, settings, new_list, undo) = ui.input(|i| (cmd(i, false, Key::K), cmd(i, false, Key::Comma), cmd(i, true, Key::N), cmd(i, false, Key::Z)));
        let (new_task, search) = ui.input(|i| (cmd(i, false, Key::N), cmd(i, false, Key::F)));
        // A printable key outside a text field starts a search in the active list (a row's own keys do the same).
        // (egui's wants_keyboard_input is true for any focused widget; a field is one with text edit state.)
        let typing = ui.memory(|m| m.focused()).is_some_and(|id| egui::text_edit::TextEditState::load(ui.ctx(), id).is_some());
        let letter = if typing || new_task || search {
            None
        } else {
            ui.input(|i| {
                if i.modifiers.command || i.modifiers.ctrl || i.modifiers.alt {
                    return None;
                }
                i.events.iter().find_map(|e| match e {
                    egui::Event::Text(t) => t.chars().next().filter(|c| c.is_alphanumeric()),
                    egui::Event::Key { key, pressed: true, .. } => {
                        let n = key.name();
                        let c = n.chars().next().filter(|_| n.chars().count() == 1).filter(|c| c.is_ascii_alphanumeric());
                        c.map(|c| if i.modifiers.shift { c } else { c.to_ascii_lowercase() })
                    }
                    _ => None,
                })
            })
        };
        let pane = app.ui.active_pane.min(1);
        if new_task {
            if let Some(list) = app.active_list() {
                app.ui.draft_list = Some(list);
            }
        }
        if search {
            ui.memory_mut(|m| m.request_focus(crate::screens::tasklist::search_id_of(app.ui.active_pane.min(1))));
        }
        if let Some(c) = letter {
            self.panes[pane].start_search(ui.ctx(), c);
        }
        if palette {
            app.ui.palette_open = true;
        }
        if settings {
            app.settings_window = Some("general".into());
        }
        if new_list {
            app.sheet = Some(crate::app::Sheet::NewList);
        }
        if undo && app.toast.as_ref().is_some_and(|t| !t.undo.is_empty()) {
            app.undo();
        }
    }
}

/// The main window's name for a sheet's accept button.
fn accept_part(sheet: &crate::app::Sheet) -> &'static str {
    use crate::app::Sheet::*;
    match sheet {
        NewList | NewProject => parts::DIALOGCREATE,
        DeleteList(_) => parts::DIALOGDELETE,
        DeleteTask(_) => parts::DIALOGMOVETOTRASH,
        _ => "app.mainwindow.dialogAccept",
    }
}

/// The same app, borrowed again for a window's content inside a closure (the closure holds the only
/// other borrow, the window's screen).
fn app_ptr(app: &mut App) -> &mut App {
    app
}

/// A region of the window as a part: a node over `rect`, named `label`.
fn node(ui: &mut Ui, rect: Rect, id: &str, part_id: &str, label: &str) -> egui::Response {
    let r = ui.interact(rect, Id::new(id), Sense::hover());
    let label = label.to_owned();
    r.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &label));
    part(ui, &r, part_id);
    r
}

/// The test's mount: the main window on the props' data set, opened where the app opens.
struct Mount {
    app: App,
    window: MainWindow,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.window.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }
    fn close(&mut self) {
        // The OS closed the frontmost window: a sheet, else Settings, else the Trash.
        if self.app.sheet.is_some() {
            self.app.sheet = None;
        } else if self.app.settings_window.is_some() {
            self.app.settings_window = None;
        } else {
            self.app.trash_window = false;
        }
    }
}

pub fn mount(app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let _p: MainwindowProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    Ok(Box::new(Mount { app, window: MainWindow::new() }))
}
