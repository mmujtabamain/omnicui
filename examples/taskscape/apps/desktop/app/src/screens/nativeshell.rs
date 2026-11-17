//! contract: omnicui/contracts/nativeshell.contract.yaml (id: app.nativeshell). The desktop shell: the menu bar (Edit, List), the tray agent's menu, the quick-capture and screenshot chords, quit and close, and the boot veil; it draws the main window and the capture bar.
use crate::screens::capturebar::CaptureBar;
use crate::screens::mainwindow::MainWindow;
use crate::App;
use egui::{pos2, vec2, Align2, Color32, CornerRadius, Event, FontFamily, FontId, Id, Key, Margin, Order, Pos2, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, UiBuilder, WidgetInfo};
use taskscape_ui::omnicui::capturebar::{parts as capture_parts, CapturebarEvent};
use taskscape_ui::omnicui::nativeshell::{parts, NativeshellEvent, NativeshellProps};
use taskscape_ui::{as_part, icon, part, rem, weight, Look, Mounted};

/// The menu bar's height: the OS menu bar's, drawn in the window.
const MENU_BAR: f32 = 24.0;
/// The capture bar's box: its width, and the height its viewport opens at (the card takes what it needs).
const BAR_W: f32 = 460.0;
const BAR_H: f32 = 120.0;
/// The id of the bar's title field: capturebar.rs gives its TextEdit this id, and focus is asked for it here.
const TITLE_FIELD: &str = "app.capturebar.title.field";

/// The main window's parts, under the shell's names.
const MAIN_RENAMES: &[(&str, &str)] = &[
    ("app.mainwindow.window", parts::MAINWINDOW),
    ("app.mainwindow.projectSwitcher", parts::PROJECTSWITCHER),
    ("app.mainwindow.listTab", parts::LISTTAB),
];

/// The capture bar's parts, under the shell's names: its card is the bar's box.
const BAR_RENAMES: &[(&str, &str)] = &[
    (capture_parts::CARD, parts::CAPTUREBAR),
    (capture_parts::TITLE, parts::CAPTURETITLE),
    (capture_parts::TARGET, parts::CAPTURETARGET),
];

/// A menu open from the menu bar.
#[derive(Clone, Copy, PartialEq, Eq)]
enum OpenMenu {
    Edit,
    List,
}

/// The desktop shell's state: what the menu bar, the tray items and the chords have set, and the capture bar.
pub struct NativeShell {
    /// The main window: the title bar, the panes, the aside and the toast.
    window: MainWindow,
    /// The quick-capture bar. Its draft lives in it; once it hides itself it is replaced (see `bar_events`).
    bar: CaptureBar,
    /// The first paint is withheld: the boot veil and box stay up.
    held: bool,
    /// The screen is drawn inside the app's own window (a test has one window): the windows and the bar are panels in it.
    in_place: bool,
    /// The main window is shown (Open Taskscape shows it).
    main_shown: bool,
    /// The main window has keyboard focus: chords go to it (decision 11).
    main_focused: bool,
    /// The capture bar is shown.
    bar_shown: bool,
    /// The bar's title had keyboard focus at the end of the last frame.
    bar_focused: bool,
    /// A reveal asks for the title's focus, applied in the bar's next frame.
    bar_focus_wanted: bool,
    /// The bar was shown at launch without focus: its title must not take focus.
    bar_unfocus_once: bool,
    /// The quick-capture combo is bound (captureHotkey).
    toggle_bound: bool,
    /// The menu open from the menu bar.
    menu: Option<OpenMenu>,
    /// The project and list last remembered, as ids; None until the first frame.
    remembered: Option<(Option<String>, Option<String>)>,
    /// The OS asked the window to close (Mounted::close); answered on the next frame.
    close_asked: bool,
}

impl NativeShell {
    pub fn new() -> Self {
        let mut window = MainWindow::new();
        window.in_place = false;
        Self {
            window,
            bar: CaptureBar::new(),
            held: false,
            in_place: false,
            main_shown: true,
            main_focused: true,
            bar_shown: false,
            bar_focused: false,
            bar_focus_wanted: false,
            bar_unfocus_once: false,
            toggle_bound: true,
            menu: None,
            remembered: None,
            close_asked: false,
        }
    }

    /// One frame of the desktop shell, filling `ui`: the chords, the close, the capture bar, the menu bar and the
    /// main window. Returns the events raised in this frame.
    pub fn show(&mut self, app: &mut App, ui: &mut Ui, look: Look) -> Vec<NativeshellEvent> {
        let mut out = vec![];
        let full = ui.max_rect();
        if self.held {
            boot_veil(ui, full, look);
            return out;
        }
        ui.painter().rect_filled(full, CornerRadius::ZERO, look.t.surface_1);
        self.chords(app, ui, &mut out);
        self.close_request(app, ui, &mut out);
        // The bar is drawn before the window, so the keys it takes are not the window's.
        if self.bar_shown {
            self.bar_ui(app, ui, full, look, &mut out);
        }
        let bar = Rect::from_min_size(full.min, vec2(full.width(), MENU_BAR));
        let anchors = self.menu_bar(app, ui, bar, look, &mut out);
        if self.main_shown {
            let body = Rect::from_min_max(pos2(full.min.x, bar.max.y), full.max);
            ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
                ui.set_clip_rect(body);
                as_part(ui, MAIN_RENAMES, |ui| self.window.show(app, ui, look));
            });
        }
        // The title is read after the window, so it shows what this frame opened.
        self.open_label(app, ui, bar, look);
        self.dropdown(app, ui, anchors, bar.bottom(), look, &mut out);
        self.remember(app, &mut out);
        if !self.in_place {
            self.window.windows(app, ui.ctx(), look);
        }
        out
    }

    /// The keys the shell takes before the bar and the window see them: Escape on the bar, the quick-capture combo
    /// and the screenshot chord. A press the shell takes is removed from the frame.
    fn chords(&mut self, app: &mut App, ui: &Ui, out: &mut Vec<NativeshellEvent>) {
        // Escape hides the bar and keeps its draft. The bar never sees the key, so it does not hide itself.
        if self.bar_shown && self.bar_focused && press(ui, Key::Escape, false, false, true) {
            self.hide_bar(app, ui.ctx());
        }
        // While the bar has focus the combo is the bar's: it submits (decision 20), so the key stays in the frame.
        let bar_has_focus = self.bar_shown && self.bar_focused;
        if self.toggle_bound && !bar_has_focus && press(ui, Key::Enter, true, false, true) {
            if self.main_focused {
                // Decision 11: the main window handles the chord, and the bar stays hidden.
                app.ui.draft_list = app.active_list();
                out.push(NativeshellEvent::ShellCommand { action: "new_task".into() });
            } else if self.bar_shown {
                self.hide_bar(app, ui.ctx());
            } else {
                self.reveal_bar(app);
            }
        }
        if press(ui, Key::Enter, true, true, true) {
            if self.main_focused {
                out.push(NativeshellEvent::ShellCommand { action: "attach_screenshot".into() });
            } else if app.data.get_setting("screenshot_mode").as_deref().unwrap_or("fullScreen") == "fullScreen" {
                // Region mode waits for the OS selection, so a hidden bar stays hidden.
                self.reveal_bar(app);
            }
        }
    }

    /// The OS close request: the close chord (Alt+F4 or Control+W, Command+W on macOS), the window's own close request, or Mounted::close.
    /// On the bar it hides the bar and the app keeps running; on the main window it quits (exit 0).
    fn close_request(&mut self, app: &mut App, ui: &Ui, out: &mut Vec<NativeshellEvent>) {
        let chord = close_chord(ui);
        let asked = std::mem::take(&mut self.close_asked);
        let os = ui.ctx().input(|i| i.viewport().close_requested());
        if !(chord || asked || os) {
            return;
        }
        if self.bar_shown && self.bar_focused {
            self.hide_bar(app, ui.ctx());
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::CancelClose);
        } else {
            out.push(NativeshellEvent::Quit { code: 0 });
        }
    }

    /// Shows the bar, with its title focused when `focus` is asked for by the reveal.
    fn reveal_bar(&mut self, app: &mut App) {
        self.bar_shown = true;
        self.bar_focus_wanted = true;
        app.capture_open = true;
    }

    /// Hides the bar and gives the title its focus back. The draft stays in the bar.
    fn hide_bar(&mut self, app: &mut App, ctx: &egui::Context) {
        self.bar_shown = false;
        self.bar_focused = false;
        self.bar_focus_wanted = false;
        app.capture_open = false;
        ctx.memory_mut(|m| m.surrender_focus(Id::new(TITLE_FIELD)));
    }

    /// The capture bar: in its own window in the app, drawn over the main window inside a test.
    fn bar_ui(&mut self, app: &mut App, ui: &Ui, full: Rect, look: Look, out: &mut Vec<NativeshellEvent>) {
        let ctx = ui.ctx().clone();
        let mut events = vec![];
        if self.in_place {
            let at = pos2(full.center().x - BAR_W / 2.0, full.top() + MENU_BAR + 12.0);
            egui::Area::new(Id::new("nativeshell.capturebar")).order(Order::Foreground).fixed_pos(at).show(&ctx, |ui| {
                ui.set_width(BAR_W);
                events = self.draw_bar(app, ui, look);
            });
        } else {
            let mut closing = false;
            let vp = egui::ViewportBuilder::default()
                .with_title("Quick capture")
                .with_inner_size([BAR_W, BAR_H])
                .with_decorations(false)
                .with_resizable(false)
                .with_always_on_top();
            ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("capturebar"), vp, |ctx, _| {
                egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| {
                    events = self.draw_bar(app, ui, look);
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    closing = true;
                }
            });
            if closing {
                self.hide_bar(app, &ctx);
            }
        }
        self.bar_events(app, &ctx, events, out);
    }

    /// The bar's card for one frame: its focus requests, the card itself, and whether the title has focus after it.
    fn draw_bar(&mut self, app: &mut App, ui: &mut Ui, look: Look) -> Vec<CapturebarEvent> {
        let title = Id::new(TITLE_FIELD);
        if std::mem::take(&mut self.bar_focus_wanted) {
            ui.ctx().memory_mut(|m| m.request_focus(title));
        }
        let events = as_part(ui, BAR_RENAMES, |ui| self.bar.show(app, ui, look));
        if std::mem::take(&mut self.bar_unfocus_once) {
            ui.ctx().memory_mut(|m| m.surrender_focus(title));
        }
        self.bar_focused = ui.ctx().memory(|m| m.has_focus(title));
        events
    }

    /// What the bar raised this frame: a submitted capture is reported and refreshes the main window; the target
    /// opens the main window; a bar that hid itself is replaced, so its next reveal starts from a fresh card.
    fn bar_events(&mut self, app: &mut App, ctx: &egui::Context, events: Vec<CapturebarEvent>, out: &mut Vec<NativeshellEvent>) {
        let mut hidden = false;
        for e in events {
            match e {
                CapturebarEvent::TaskSaved { title, .. } => {
                    out.push(NativeshellEvent::CaptureSubmitted { title });
                    out.push(NativeshellEvent::ShellCommand { action: "refresh".into() });
                }
                CapturebarEvent::MainRequested { .. } => {
                    self.main_shown = true;
                    self.main_focused = true;
                }
                CapturebarEvent::BarHidden { .. } => hidden = true,
                _ => {}
            }
        }
        if hidden {
            self.hide_bar(app, ctx);
            self.bar = CaptureBar::new();
        }
    }

    /// The menu bar: the Edit and List titles at the left, and the tray agent's items at the right (its tooltip icon,
    /// Open Taskscape, Quit Taskscape). Returns the left x of the Edit and List titles, where their menus open.
    fn menu_bar(&mut self, app: &mut App, ui: &mut Ui, rect: Rect, look: Look, out: &mut Vec<NativeshellEvent>) -> [f32; 2] {
        let t = look.t;
        ui.painter().rect_filled(rect, CornerRadius::ZERO, t.surface_1);
        ui.painter().rect_filled(Rect::from_min_size(pos2(rect.left(), rect.bottom() - 1.0), vec2(rect.width(), 1.0)), CornerRadius::ZERO, t.edge_1);
        node(ui, rect, "nativeshell.menubar", parts::MENUBAR, "Menu bar");

        let (edit_rect, edit) = bar_word(ui, look, pos2(rect.left() + 4.0, rect.top()), "Edit", parts::MENUEDIT);
        let (list_rect, list) = bar_word(ui, look, pos2(edit_rect.right(), rect.top()), "List", parts::MENULIST);
        for (pressed, menu) in [(edit.clicked(), OpenMenu::Edit), (list.clicked(), OpenMenu::List)] {
            if pressed {
                self.menu = if self.menu == Some(menu) { None } else { Some(menu) };
            }
        }

        let quit_w = word_width(ui, "Quit Taskscape");
        let open_w = word_width(ui, "Open Taskscape");
        let mut right = rect.right() - 4.0;
        let (_, quit) = bar_word(ui, look, pos2(right - quit_w, rect.top()), "Quit Taskscape", parts::TRAYQUIT);
        right -= quit_w;
        let (_, open) = bar_word(ui, look, pos2(right - open_w, rect.top()), "Open Taskscape", parts::TRAYOPEN);
        right -= open_w;
        self.tray_icon(ui, look, Rect::from_min_size(pos2(right - 24.0, rect.top()), vec2(24.0, MENU_BAR)));
        if open.clicked() {
            self.main_shown = true;
            self.main_focused = true;
        }
        if quit.clicked() {
            out.push(NativeshellEvent::Quit { code: 0 });
        }
        [edit_rect.left(), list_rect.left()]
    }

    /// The tray agent's icon: its tooltip is the status the agent sets (the capture combo, or plain when unbound).
    fn tray_icon(&self, ui: &mut Ui, look: Look, rect: Rect) {
        let tip = self.tray_tooltip();
        let r = ui.interact(rect, Id::new(parts::TRAYTOOLTIP), Sense::hover()).on_hover_text(tip.clone());
        ui.painter().text(rect.center(), Align2::CENTER_CENTER, icon("task_alt", 15.0).text(), FontId::new(15.0, FontFamily::Name("Icons".into())), look.t.content_2);
        r.widget_info(|| WidgetInfo::labeled(Role::Status, true, tip.as_str()));
        ui.ctx().accesskit_node_builder(r.id, |n| n.set_value(tip.clone()));
        part(ui, &r, parts::TRAYTOOLTIP);
    }

    fn tray_tooltip(&self) -> String {
        if self.toggle_bound {
            let combo = if cfg!(target_os = "macos") { "⌘Enter" } else { "Ctrl Enter" };
            format!("Taskscape: {combo} to capture")
        } else {
            "Taskscape".into()
        }
    }

    /// The project and list the main window has open, in the middle of the menu bar.
    fn open_label(&self, app: &App, ui: &mut Ui, bar: Rect, look: Look) {
        let text = open_text(app);
        let area = Rect::from_center_size(bar.center(), vec2(280.0, MENU_BAR));
        let resp = ui.interact(area, Id::new(parts::OPENLISTLABEL), Sense::hover());
        ui.painter().text(area.center(), Align2::CENTER_CENTER, text.as_str(), FontId::new(rem(0.857), weight(500)), look.t.content_2);
        resp.widget_info(|| WidgetInfo::labeled(Role::Status, true, text.as_str()));
        ui.ctx().accesskit_node_builder(resp.id, |n| n.set_value(text.clone()));
        part(ui, &resp, parts::OPENLISTLABEL);
    }

    /// The open menu's rows, drawn under its title, and the pick the press makes. Escape closes it.
    fn dropdown(&mut self, app: &mut App, ui: &mut Ui, anchors: [f32; 2], y: f32, look: Look, out: &mut Vec<NativeshellEvent>) {
        let Some(menu) = self.menu else { return };
        if press(ui, Key::Escape, false, false, true) {
            self.menu = None;
            return;
        }
        let t = look.t;
        let ctx = ui.ctx().clone();
        let at = pos2(if menu == OpenMenu::Edit { anchors[0] } else { anchors[1] }, y);
        let rows: &[(&str, &str)] = match menu {
            OpenMenu::Edit => &[("Undo", "")],
            OpenMenu::List => &[("Import List from JSON…", parts::MENUIMPORTLIST), ("Export List to JSON…", parts::MENUEXPORTLIST)],
        };
        let mut picked: Option<usize> = None;
        egui::Area::new(Id::new("nativeshell.menu")).order(Order::Foreground).fixed_pos(at).show(&ctx, |ui| {
            egui::Frame::new()
                .fill(t.surface_2)
                .stroke(Stroke::new(1.0, t.edge_2))
                .corner_radius(CornerRadius::same(7))
                .inner_margin(Margin::same(4))
                .show(ui, |ui| {
                    ui.set_width(232.0);
                    for (i, (label, part_id)) in rows.iter().enumerate() {
                        let (rect, resp) = ui.allocate_exact_size(vec2(224.0, 26.0), Sense::click());
                        if resp.hovered() {
                            ui.painter().rect_filled(rect, CornerRadius::same(4), t.wash_2);
                        }
                        let galley = ui.painter().layout_no_wrap(label.to_string(), FontId::new(rem(0.929), weight(400)), t.content_1);
                        let y = rect.center().y - galley.size().y / 2.0;
                        ui.painter().galley(pos2(rect.left() + 10.0, y), galley, t.content_1);
                        resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, *label));
                        if !part_id.is_empty() {
                            part(ui, &resp, part_id);
                        }
                        if resp.clicked() {
                            picked = Some(i);
                        }
                    }
                });
        });
        if let Some(i) = picked {
            self.menu = None;
            match menu {
                OpenMenu::Edit => {
                    if app.toast.as_ref().is_some_and(|toast| !toast.undo.is_empty()) {
                        app.undo();
                    }
                }
                OpenMenu::List => {
                    let action = if i == 0 { "import_list" } else { "export_list" };
                    out.push(NativeshellEvent::ShellCommand { action: action.into() });
                }
            }
        }
    }

    /// Remembers the project and list the window has open when they change, and reports each change once.
    fn remember(&mut self, app: &mut App, out: &mut Vec<NativeshellEvent>) {
        let now = (app.ui.project.clone(), app.active_list());
        if let Some(prev) = &self.remembered {
            if *prev != now {
                app.remember();
                let project = now.0.as_ref().and_then(|p| app.data.project(p)).map(|p| p.name.clone()).unwrap_or_default();
                let list = now.1.as_ref().and_then(|l| app.data.list(l)).map(|l| l.name.clone()).unwrap_or_default();
                out.push(NativeshellEvent::Remembered { list, project });
            }
        }
        self.remembered = Some(now);
    }
}

/// Whether a key was pressed this frame with exactly these modifiers: the platform modifier (Command on macOS,
/// Control elsewhere, either flag counts) when `platform` is set, and Shift when `shift` is set. With `take`, the
/// press is removed from the frame's input, so no later widget sees it.
fn press(ui: &Ui, key: Key, platform: bool, shift: bool, take: bool) -> bool {
    ui.input_mut(|i| {
        let at = i.events.iter().position(|e| match e {
            Event::Key { key: k, modifiers: m, pressed: true, .. } => {
                let cmd = if cfg!(target_os = "macos") { m.command || m.mac_cmd } else { m.ctrl || m.command };
                *k == key && cmd == platform && m.shift == shift && !m.alt
            }
            _ => false,
        });
        match at {
            Some(at) => {
                if take {
                    i.events.remove(at);
                }
                true
            }
            None => false,
        }
    })
}

/// Whether the window close chord was pressed this frame, and takes it: Alt+F4 or Control+W on Windows and Linux,
/// Command+W on macOS. The chord is the OS close request the window answers (decision 2).
fn close_chord(ui: &Ui) -> bool {
    ui.input_mut(|i| {
        let at = i.events.iter().position(|e| match e {
            Event::Key { key, modifiers: m, pressed: true, .. } => {
                if cfg!(target_os = "macos") {
                    *key == Key::W && (m.command || m.mac_cmd) && !m.shift && !m.alt
                } else {
                    (*key == Key::F4 && m.alt) || (*key == Key::W && m.ctrl)
                }
            }
            _ => false,
        });
        match at {
            Some(at) => {
                i.events.remove(at);
                true
            }
            None => false,
        }
    })
}

/// A region of the window as a part: a node over `rect`, named `label`.
fn node(ui: &mut Ui, rect: Rect, id: &str, part_id: &str, label: &str) {
    let r = ui.interact(rect, Id::new(id), Sense::hover());
    r.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, label));
    part(ui, &r, part_id);
}

/// A word of the menu bar at `at`: a button as wide as its text. It takes a wash on hover and carries the part.
fn bar_word(ui: &mut Ui, look: Look, at: Pos2, label: &str, part_id: &str) -> (Rect, Response) {
    let t = look.t;
    let galley = ui.painter().layout_no_wrap(label.to_owned(), FontId::new(rem(0.857), weight(500)), t.content_1);
    let rect = Rect::from_min_size(at, vec2(galley.size().x + 16.0, MENU_BAR));
    let resp = ui.interact(rect, Id::new(part_id), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect.shrink(2.0), CornerRadius::same(4), t.wash_1);
    }
    let y = rect.center().y - galley.size().y / 2.0;
    ui.painter().galley(pos2(rect.left() + 8.0, y), galley, t.content_1);
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    part(ui, &resp, part_id);
    (rect, resp)
}

/// The width a menu-bar word takes, as `bar_word` lays it out.
fn word_width(ui: &Ui, label: &str) -> f32 {
    ui.painter().layout_no_wrap(label.to_owned(), FontId::new(rem(0.857), weight(500)), Color32::BLACK).size().x + 16.0
}

/// The project and list the window has open, as the title reads them ("Home · Errands").
fn open_text(app: &App) -> String {
    let project = app.ui.project.as_ref().and_then(|p| app.data.project(p)).map(|p| p.name.clone()).unwrap_or_default();
    let list = app.active_list().and_then(|l| app.data.list(&l)).map(|l| l.name.clone()).unwrap_or_default();
    match (project.is_empty(), list.is_empty()) {
        (false, false) => format!("{project} · {list}"),
        (false, true) => project,
        (true, false) => list,
        (true, true) => "Taskscape".into(),
    }
}

/// The boot veil over the window until the first paint, with the empty box at its centre (no tick is recorded).
fn boot_veil(ui: &mut Ui, full: Rect, look: Look) {
    ui.painter().rect_filled(full, CornerRadius::ZERO, look.t.surface_1);
    node(ui, full, "nativeshell.bootveil", parts::BOOTVEIL, "Starting Taskscape");
    let b = Rect::from_center_size(full.center(), vec2(25.0, 25.0));
    let r = ui.interact(b, Id::new("nativeshell.bootbox"), Sense::hover());
    ui.painter().rect_stroke(b, CornerRadius::same(4), Stroke::new(1.0, look.t.content_3), StrokeKind::Inside);
    r.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, false, "Starting Taskscape"));
    part(ui, &r, parts::BOOTBOX);
}

/// The test's mount: the app on the props' data set, the shell's state as the props set it.
struct Mount {
    app: App,
    screen: NativeShell,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
    fn close(&mut self) {
        // The OS asked the window to close: the next frame answers it (the bar hides, or the app quits).
        self.screen.close_asked = true;
    }
}

pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: NativeshellProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    // The remembered project and list, stored before launch: names are looked up to their ids. A name that is not
    // found is stored as it is, and the app falls back to its first project or list.
    if !p.lastProject.is_empty() {
        let id = app.data.project_named(&p.lastProject).map(|x| x.id.clone()).unwrap_or_else(|| p.lastProject.clone());
        app.data.set_setting("last_active_project", &id).map_err(|e| e.to_string())?;
    }
    if !p.lastList.is_empty() {
        let project = app.data.project_named(&p.lastProject).map(|x| x.id.clone());
        let id = app.data.list_named(project.as_deref(), &p.lastList).map(|l| l.id.clone()).unwrap_or_else(|| p.lastList.clone());
        app.data.set_setting("last_active_list", &id).map_err(|e| e.to_string())?;
    }
    if !p.lastProject.is_empty() || !p.lastList.is_empty() {
        app.open_last();
    }
    app.data.set_setting("screenshot_mode", &p.captureMode).map_err(|e| e.to_string())?;

    let mut screen = NativeShell::new();
    screen.in_place = true;
    screen.window.in_place = true;
    screen.held = p.boot == "held";
    screen.main_shown = p.open != "background";
    screen.main_focused = p.focus == "main" && screen.main_shown;
    screen.toggle_bound = p.captureHotkey != "unbound";
    if p.barState == "shown" {
        screen.bar_shown = true;
        screen.bar_unfocus_once = true;
        app.capture_open = true;
    }
    Ok(Box::new(Mount { app, screen }))
}
