//! contract: omnicui/contracts/settingsshortcuts.contract.yaml (id: app.settingsshortcuts). The Settings window on its Shortcuts page: the title bar, the section list with search, and the Main window's shortcut rows with their keycaps.
use crate::App;
use egui::{Align, Color32, CornerRadius, FontId, Id, Key, Layout, Modifiers, Rect, Role, ScrollArea, Sense, Stroke, UiBuilder, WidgetInfo, pos2, vec2};
use taskscape_ui::omnicui::button::{parts as button_parts, ButtonEvent, ButtonProps};
use taskscape_ui::omnicui::emptystate::EmptystateProps;
use taskscape_ui::omnicui::heading::{parts as heading_parts, HeadingProps};
use taskscape_ui::omnicui::icon::IconProps;
use taskscape_ui::omnicui::input::{parts as input_parts, InputEvent, InputProps};
use taskscape_ui::omnicui::keycap::{parts as keycap_parts, KeycapEvent, KeycapProps};
use taskscape_ui::omnicui::settingsshortcuts::{parts, SettingsshortcutsEvent, SettingsshortcutsProps};
use taskscape_ui::omnicui::sidebartab::{parts as sidebartab_parts, SidebartabProps};
use taskscape_ui::omnicui::text::TextProps;
use taskscape_ui::widgets::button::Button;
use taskscape_ui::widgets::emptystate::Emptystate;
use taskscape_ui::widgets::heading::Heading;
use taskscape_ui::widgets::icon::Icon;
use taskscape_ui::widgets::input::Input;
use taskscape_ui::widgets::keycap::Keycap;
use taskscape_ui::widgets::sidebartab::Sidebartab;
use taskscape_ui::widgets::text::Text;
use taskscape_ui::{as_part, part, rem, weight, Look, Mounted};

/// The desktop keeps the reference's Mod chords for the commands the web remaps (decision 18).
const MAC: bool = cfg!(target_os = "macos");
/// The chords the OS keeps for itself, in the keycap's notation (decision 23). Windows' Win chords are not
/// visible to egui, so they cannot be refused here.
const RESERVED: &str = if MAC { "Meta+Q,Meta+H,Meta+M,Meta+Tab,Meta+Space" } else { "Alt+F4,Alt+Tab" };

const GROUP: &str = "Main window";
const INTRO: &str = "Click a shortcut to change it — press the new keys, Backspace to remove, Esc to cancel.";
/// The id egui gives the search field's text editor (the Input widget's field).
const SEARCH_FIELD: &str = "ui.input.field";

const HEADER_H: f32 = rem(3.286);
const HEADER_PAD_L: f32 = rem(1.286);
const HEADER_PAD_R: f32 = rem(0.571);
const NAV_W: f32 = rem(10.929);
const NAV_PAD: f32 = rem(0.5);
const WELL_H: f32 = rem(1.786);
const TAB_W: f32 = rem(9.929);
const TAB_H: f32 = rem(2.143);
const PANE_PAD_X: f32 = rem(1.286);
const PANE_PAD_TOP: f32 = rem(0.857);
const PANE_PAD_BOTTOM: f32 = rem(1.286);
const BLOCK_GAP: f32 = rem(1.214);
const ROW_H: f32 = rem(2.5);
const ROW_PAD_L: f32 = rem(0.571);
const ROW_PAD_R: f32 = rem(0.714);
/// A small icon button is a square of this size (the web's h-21 at 14 px).
const ICON_BUTTON: f32 = 21.0;
const GAP: f32 = 8.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pane {
    General,
    Shortcuts,
    About,
}

/// The section list, in the web's order: the label and the icon of each tab.
const PANES: [(Pane, &str, &str); 3] = [(Pane::General, "General", "tune"), (Pane::Shortcuts, "Shortcuts", "keyboard"), (Pane::About, "About", "info")];

/// One row of the Main window group: the command, its title, the desktop's default chord (the reference chords)
/// and the parts it carries (the web's row and keycap parts).
struct Row {
    command: &'static str,
    label: &'static str,
    accel: &'static str,
    row: Option<&'static str>,
    key: Option<&'static str>,
}

const fn r(command: &'static str, label: &'static str, accel: &'static str, row: Option<&'static str>, key: Option<&'static str>) -> Row {
    Row { command, label, accel, row, key }
}

/// The Main window group in the catalog's order (31 rows).
const MAIN_ROWS: &[Row] = &[
    r("new_task", "New task", "Mod+N", Some(parts::ROWNEWTASK), Some(parts::KEYCAPNEWTASK)),
    r("search", "Search", "Mod+F", None, Some(parts::KEYCAPSEARCH)),
    r("toggle_preview", "Toggle preview", "Mod+\\", None, None),
    r("open_settings", "Open settings", "Mod+,", None, None),
    r("delete_task", "Delete task", "Mod+Backspace", None, None),
    r("switch_list_1", "Switch to list 1", "Mod+1", None, Some(parts::KEYCAPSWITCHLIST1)),
    r("switch_list_2", "Switch to list 2", "Mod+2", None, None),
    r("switch_list_3", "Switch to list 3", "Mod+3", None, None),
    r("switch_list_4", "Switch to list 4", "Mod+4", None, None),
    r("switch_list_5", "Switch to list 5", "Mod+5", None, None),
    r("switch_list_6", "Switch to list 6", "Mod+6", None, None),
    r("switch_list_7", "Switch to list 7", "Mod+7", None, None),
    r("switch_list_8", "Switch to list 8", "Mod+8", None, None),
    r("switch_list_9", "Switch to list 9", "Mod+9", None, None),
    r("prev_tab", "Previous list tab", "Mod+Alt+ArrowLeft", None, Some(parts::KEYCAPPREVTAB)),
    r("next_tab", "Next list tab", "Mod+Alt+ArrowRight", None, None),
    r("prev_project", "Previous project", "Mod+Shift+[", None, Some(parts::KEYCAPPREVPROJECT)),
    r("next_project", "Next project", "Mod+Shift+]", None, None),
    r("move_up", "Move task up", "Mod+Alt+ArrowUp", None, None),
    r("move_down", "Move task down", "Mod+Alt+ArrowDown", None, None),
    r("select_all", "Select all tasks", "Mod+A", None, None),
    r("command_palette", "Command palette", "Mod+K", None, None),
    r("toggle_split", "Toggle split view", "Mod+Alt+S", None, None),
    r("focus_other_pane", "Focus other pane", "Mod+Alt+O", None, None),
    r("new_list", "New list", "Mod+Shift+N", Some(parts::ROWNEWLIST), Some(parts::KEYCAPNEWLIST)),
    r("new_project", "New project", "Mod+Shift+P", Some(parts::ROWNEWPROJECT), None),
    r("show_completed", "Show/hide completed", "Mod+Shift+H", None, None),
    r("collapse_all", "Collapse all", "Mod+Alt+-", None, None),
    r("expand_all", "Expand all", "Mod+Alt+=", None, None),
    r("undo", "Undo", "Mod+Z", Some(parts::ROWUNDO), Some(parts::KEYCAPUNDO)),
    r("redo", "Redo", "Mod+Shift+Z", Some(parts::ROWREDO), Some(parts::KEYCAPREDO)),
];

/// What a row's keycap or reset control asked for, applied after the frame is drawn.
enum Action {
    Start(usize),
    Cancel(usize),
    Capture(usize, String),
    Refused(usize, String),
    Reset(usize),
    Discard(Vec<usize>),
}

/// What the frame shows, worked out before drawing.
struct View {
    searching: bool,
    /// The query, trimmed (the empty state quotes it).
    query: String,
    /// The rows that match (all of them when there is no search), in order.
    rows: Vec<usize>,
    /// The rows whose accel differs from the one they had when this window opened.
    changed: Vec<usize>,
}

/// The Settings window's Shortcuts page: the title bar, the section list with the search field, and the Main
/// window rows. Each change is saved as it is made and reported as a bindingChanged event.
pub struct SettingsShortcuts {
    /// Settings is open. Escape with no search closes it.
    open: bool,
    /// Drawn inside the Settings window, which has its own sidebar: the page stays on its own section and
    /// ignores its sidebar, which sits out of view.
    pub embedded: bool,
    /// The saved bindings have been read from the app's data (on the first frame).
    loaded: bool,
    pane: Pane,
    query: String,
    /// The search was scoped by choosing a section that has matches: the group heading drops its prefix.
    scoped: bool,
    /// The row whose keycap is recording a new combo.
    recording: Option<usize>,
    /// Each row's saved accel, in MAIN_ROWS order (the stored form: Mod, Alt, Shift, then the key).
    accels: Vec<String>,
    /// Each row's accel when this window first showed: what the undo controls restore.
    opened: Vec<String>,
    /// The refusal or failure line under each row.
    errors: Vec<String>,
    /// Each row's keycap, with the mode and accel it was built for. It is rebuilt when either changes.
    keys: Vec<Option<(String, String, Keycap)>>,
    pane_key: (Pane, bool),
    /// Changes whenever the pane or the search scope changes, so the scroll area starts at the top.
    pane_gen: u32,
    last_focus: Option<Id>,
    /// The search field takes the keyboard on the first frame, when Settings opens.
    autofocus: bool,
}

impl SettingsShortcuts {
    pub fn new() -> Self {
        let n = MAIN_ROWS.len();
        let accels: Vec<String> = MAIN_ROWS.iter().map(|row| row.accel.to_owned()).collect();
        Self {
            open: true,
            embedded: false,
            loaded: false,
            pane: Pane::Shortcuts,
            query: String::new(),
            scoped: false,
            recording: None,
            opened: accels.clone(),
            accels,
            errors: vec![String::new(); n],
            keys: (0..n).map(|_| None).collect(),
            pane_key: (Pane::Shortcuts, false),
            pane_gen: 0,
            last_focus: None,
            autofocus: true,
        }
    }

    /// Whether Settings is still open: Escape with no search, or the OS closing its window, ends it.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// One frame: the Settings window's content on the Shortcuts page, and the events it raised in this frame.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<SettingsshortcutsEvent> {
        let mut events = Vec::new();
        if !self.loaded {
            self.load(app);
        }
        if !self.open {
            return events;
        }
        if self.embedded {
            self.pane = Pane::Shortcuts;
        }
        let t = look.t;
        let search_id = Id::new(SEARCH_FIELD);

        // Escape: a recording keycap takes it (it cancels the recording); otherwise it clears the search, and with
        // no search it closes Settings.
        if self.recording.is_none() && ui.input(|i| i.key_pressed(Key::Escape)) {
            if self.query.is_empty() {
                self.open = false;
                return events;
            }
            self.clear_query();
        }

        let tokens = query_tokens(&self.query);
        let searching = !tokens.is_empty();
        let rows: Vec<usize> = (0..MAIN_ROWS.len()).filter(|&i| !searching || row_matches(&MAIN_ROWS[i], &tokens, &self.accels[i])).collect();
        let changed: Vec<usize> = (0..MAIN_ROWS.len()).filter(|&i| self.accels[i] != self.opened[i]).collect();
        let count_shortcuts = if searching { rows.len() } else { 0 };
        let view = View { searching, query: self.query.trim().to_owned(), rows, changed };

        let pane_key = (self.pane, searching);
        if pane_key != self.pane_key {
            self.pane_key = pane_key;
            self.pane_gen += 1;
        }

        let mut actions: Vec<Action> = Vec::new();
        let mut typed: Option<String> = None;
        let mut clear_clicked = false;

        // The window: its background, then the title bar.
        let rect = ui.max_rect();
        let root = ui.interact(rect, ui.id().with("settings-dialog"), Sense::hover());
        root.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Settings"));
        part(ui, &root, parts::DIALOG);
        ui.painter().rect_filled(rect, CornerRadius::same(0), t.surface_2);

        let header = Rect::from_min_size(rect.min, vec2(rect.width(), HEADER_H));
        ui.painter().rect_filled(header, CornerRadius::same(0), t.surface_1);
        ui.painter().line_segment([header.left_bottom(), header.right_bottom()], Stroke::new(1.0, t.edge_1));
        let title_rect = Rect::from_min_max(pos2(header.left() + HEADER_PAD_L, header.top()), pos2(header.right() - HEADER_PAD_R, header.bottom()));
        let mut title_ui = ui.new_child(UiBuilder::new().max_rect(title_rect).layout(Layout::left_to_right(Align::Center)));
        let mut heading = Heading::new(HeadingProps { r#as: "h1".into(), content: "Settings".into(), size: "sm".into(), overflow: "truncate".into(), ..Default::default() });
        as_part(&mut title_ui, &[(heading_parts::TITLE, parts::TITLE)], |ui| heading.ui(ui, look));
        if !view.changed.is_empty() {
            title_ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let props = ButtonProps {
                    iconSize: 19 as _,
                    ..icon_button_props("undo", "xl", "strong", "Discard changes", "Discard changes made since this window opened")
                };
                if icon_button(ui, look, parts::DISCARDALL, props) {
                    actions.push(Action::Discard(view.changed.clone()));
                }
            });
        }

        // The body: the section list on the left, the pane on the right.
        let body = Rect::from_min_max(pos2(rect.left(), header.bottom()), rect.max);
        let nav = Rect::from_min_size(body.min, vec2(NAV_W, body.height()));
        let pane = Rect::from_min_max(pos2(nav.right(), body.top()), body.max);
        ui.painter().rect_filled(nav, CornerRadius::same(0), t.surface_1);
        ui.painter().line_segment([nav.right_top(), nav.right_bottom()], Stroke::new(1.0, t.edge_1));

        let mut nav_ui = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_max(nav.min + vec2(NAV_PAD, NAV_PAD), nav.max - vec2(NAV_PAD, 0.0))).layout(Layout::top_down(Align::Min)));
        // While the search has focus, Tab is kept for the handler below: it lands on the selected section.
        let search_had_focus = ui.memory(|m| m.has_focus(search_id));
        if search_had_focus {
            ui.memory_mut(|m| m.set_focus_lock_filter(search_id, egui::EventFilter { tab: true, ..Default::default() }));
        }

        if self.autofocus {
            self.autofocus = false;
            ui.memory_mut(|m| m.request_focus(search_id));
        }

        // The search well: a search glyph, the field, and a clear button while the query is not empty.
        let has_clear = !view.query.is_empty() || !self.query.is_empty();
        let query_now = self.query.clone();
        let well_w = nav_ui.available_width();
        nav_ui.allocate_ui_with_layout(vec2(well_w, WELL_H), Layout::left_to_right(Align::Center), |well| {
            well.painter().rect_filled(well.max_rect(), CornerRadius::same(4), t.wash_2);
            well.add_space(rem(0.643));
            let mut glyph = Icon::new(IconProps {
                name: "search".into(),
                size: "sm".into(),
                tone: "muted".into(),
                tint: "none".into(),
                reveal: "always".into(),
                rotate: "none".into(),
                label: String::new(),
                ..Default::default()
            });
            glyph.ui(well, look);
            well.add_space(rem(0.375));
            let clear_w = if has_clear { ICON_BUTTON + rem(0.143) } else { 0.0 };
            let field_w = (well.available_width() - clear_w - rem(0.143)).max(0.0);
            well.allocate_ui_with_layout(vec2(field_w, WELL_H), Layout::left_to_right(Align::Center), |field| {
                let mut input = Input::new(InputProps {
                    value: query_now.clone(),
                    mode: "bare".into(),
                    size: "sm".into(),
                    placeholder: "Search".into(),
                    ariaLabel: "Search settings".into(),
                    autoFocus: false,
                    selectOnOpen: false,
                    selectOnFocus: false,
                    multiline: false,
                    disabled: false,
                    ..Default::default()
                });
                let events = as_part(field, &[(input_parts::FIELD, parts::SEARCH)], |ui| input.ui(ui, look));
                for event in events {
                    if let InputEvent::ValueChange { value } = event {
                        typed = Some(value);
                    }
                }
            });
            if has_clear {
                let props = ButtonProps {
                    iconSize: 14 as _,
                    ..icon_button_props("close", "sm", "muted", "Clear search", "Clear search")
                };
                if icon_button(well, look, parts::SEARCHCLEAR, props) {
                    clear_clicked = true;
                }
            }
        });
        nav_ui.add_space(rem(0.571));

        // The sections: each a tab of the vertical list. Arrows move between them and wrap; a press selects.
        let mut tab_ids: Vec<Id> = Vec::with_capacity(PANES.len());
        let mut focused_tab: Option<usize> = None;
        for (k, &(pane_id, label, icon_name)) in PANES.iter().enumerate() {
            let id = nav_ui.next_auto_id();
            tab_ids.push(id);
            if nav_ui.memory(|m| m.has_focus(id)) {
                focused_tab = Some(k);
                nav_ui.memory_mut(|m| m.set_focus_lock_filter(id, egui::EventFilter { vertical_arrows: true, ..Default::default() }));
            }
            let count = if pane_id == Pane::Shortcuts { count_shortcuts } else { 0 };
            let tab_rect = Rect::from_min_size(nav_ui.cursor().min, vec2(nav_ui.available_width().min(TAB_W), TAB_H));
            let mut tab = Sidebartab::new(SidebartabProps {
                label: label.to_owned(),
                icon: icon_name.to_owned(),
                selected: self.pane == pane_id,
                scoped: !searching || self.scoped,
                dimmed: searching && count == 0,
                badge: count as _,
                ..Default::default()
            });
            let mut renames = vec![(sidebartab_parts::TAB, tab_part(pane_id))];
            if pane_id == Pane::Shortcuts {
                renames.push((sidebartab_parts::BADGE, parts::SHORTCUTSMATCHES));
            }
            let events = as_part(&mut nav_ui, &renames, |ui| tab.ui(ui, look));
            let pressed = !self.embedded && nav_ui.input(|i| i.pointer.primary_clicked() && i.pointer.interact_pos().is_some_and(|p| tab_rect.contains(p)));
            if !events.is_empty() || pressed {
                self.choose(pane_id, count, searching);
            }
            if k + 1 < PANES.len() {
                nav_ui.add_space(rem(0.143));
            }
        }
        if let Some(k) = focused_tab {
            let down = ui.input(|i| i.key_pressed(Key::ArrowDown));
            let up = ui.input(|i| i.key_pressed(Key::ArrowUp));
            let n = PANES.len();
            let next = if down {
                Some((k + 1) % n)
            } else if up {
                Some((k + n - 1) % n)
            } else {
                None
            };
            if let Some(next) = next {
                ui.memory_mut(|m| m.request_focus(tab_ids[next]));
                let pane_id = PANES[next].0;
                self.choose(pane_id, if pane_id == Pane::Shortcuts { count_shortcuts } else { 0 }, searching);
            }
        }
        if search_had_focus && ui.input(|i| i.key_pressed(Key::Tab)) {
            let _ = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Tab));
            let selected = PANES.iter().position(|p| p.0 == self.pane).unwrap_or(1);
            ui.memory_mut(|m| m.request_focus(tab_ids[selected]));
        }

        // The pane: the scrolling content to the right of the list.
        let pane_name = PANES.iter().find(|p| p.0 == self.pane).map_or("", |p| p.1);
        let pane_node = ui.interact(pane, ui.id().with("settings-pane"), Sense::hover());
        pane_node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, pane_name));
        part(ui, &pane_node, parts::PANE);
        let pane_left = pane.left();
        let mut pane_ui = ui.new_child(UiBuilder::new().max_rect(pane).layout(Layout::top_down(Align::Min)));
        ScrollArea::vertical()
            .id_salt(("settings-pane", self.pane_gen))
            .auto_shrink([false, false])
            .show(&mut pane_ui, |ui| {
                ui.add_space(PANE_PAD_TOP);
                ui.horizontal(|ui| {
                    ui.add_space(PANE_PAD_X);
                    ui.vertical(|ui| {
                        ui.set_width((ui.available_width() - PANE_PAD_X).max(0.0));
                        self.content(ui, look, &view, &mut actions);
                    });
                });
                ui.add_space(PANE_PAD_BOTTOM);
                // Focusing a control in the pane (a keycap, say) scrolls it into view.
                let focus = ui.memory(|m| m.focused());
                if focus != self.last_focus {
                    self.last_focus = focus;
                    if let Some(response) = focus.and_then(|f| ui.ctx().read_response(f)) {
                        if response.rect.left() >= pane_left - 0.5 {
                            ui.scroll_to_rect(response.rect, Some(Align::Center));
                        }
                    }
                }
            });

        // Changes made while drawing take effect now.
        if let Some(value) = typed {
            self.query = value;
            self.scoped = false;
        }
        if clear_clicked {
            self.clear_query();
            ui.memory_mut(|m| m.request_focus(search_id));
        }
        for action in actions {
            self.apply(app, action, &mut events);
        }
        events
    }

    /// The Shortcuts page's content: the intro, the Main window group with its rows, or the empty result.
    fn content(&mut self, ui: &mut egui::Ui, look: Look, view: &View, actions: &mut Vec<Action>) {
        let t = look.t;
        if view.searching && view.rows.is_empty() {
            let top = ui.cursor().top();
            let mut empty = Emptystate::new(EmptystateProps {
                scope: "list".into(),
                title: "No settings match".into(),
                subtitle: format!("Nothing here for “{}”.", view.query),
                icon: "search".into(),
                iconSize: 30 as _,
                ..Default::default()
            });
            let _ = empty.ui(ui, look);
            let bottom = ui.cursor().top();
            let full = ui.max_rect();
            // The empty state is one status node, so its text is the whole message.
            let area = Rect::from_min_max(pos2(full.left(), top), pos2(full.right(), bottom));
            let node = ui.interact(area, ui.id().with("no-match"), Sense::hover());
            let label = format!("No settings match. Nothing here for “{}”.", view.query);
            node.widget_info(|| WidgetInfo::labeled(Role::Status, true, label.as_str()));
            part(ui, &node, parts::NOMATCH);
            return;
        }
        // The other sections are drawn by their own screens; only the Shortcuts page (or a search) shows rows here.
        if self.pane != Pane::Shortcuts && !view.searching {
            return;
        }

        if !view.searching {
            paragraph(ui, INTRO, rem(0.857), 400, t.content_3, parts::INTRO);
            ui.add_space(BLOCK_GAP);
        }

        let heading = if view.searching && !self.scoped { "Shortcuts · Main window" } else { GROUP };
        let changed = !view.changed.is_empty();
        ui.horizontal(|ui| {
            group_label(ui, look, heading);
            let trailing = if changed { ICON_BUTTON + GAP } else { 0.0 };
            let line_w = (ui.available_width() - trailing - GAP).max(0.0);
            let (line, _) = ui.allocate_exact_size(vec2(line_w, 1.0), Sense::hover());
            ui.painter().line_segment([line.left_center(), line.right_center()], Stroke::new(1.0, t.edge_2));
            if changed {
                let props = ButtonProps {
                    iconSize: 14 as _,
                    ..icon_button_props("undo", "sm", "muted", "Discard changes in this group", "Discard changes made since this window opened")
                };
                if icon_button(ui, look, parts::DISCARDGROUP, props) {
                    actions.push(Action::Discard(view.changed.clone()));
                }
            }
        });

        ui.add_space(rem(0.5));
        egui::Frame::NONE
            .fill(t.surface_2)
            .stroke(Stroke::new(1.0, t.edge_2))
            .corner_radius(CornerRadius::same(10))
            .show(ui, |ui| {
                for (n, &i) in view.rows.iter().enumerate() {
                    if n > 0 {
                        let (line, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                        ui.painter().rect_filled(line, CornerRadius::same(0), t.edge_1);
                    }
                    self.row(ui, look, i, actions);
                }
            });
    }

    /// One shortcut row: the title on the left, the keycap on the right, the reset beside it while the row differs
    /// from its default, and the refusal line under it.
    fn row(&mut self, ui: &mut egui::Ui, look: Look, i: usize, actions: &mut Vec<Action>) {
        let item = &MAIN_ROWS[i];
        let accel = self.accels[i].clone();
        let recording = self.recording == Some(i);
        let mode = if recording { "recording" } else { "idle" };
        let stored = keycap_accel(&accel);
        let show_reset = !recording && accel != item.accel;

        // The row's own node is added before its controls, so they stay on top of it for hit tests.
        let row_rect = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), ROW_H));
        let node = ui.interact(row_rect, ui.id().with(("settings-row", i)), Sense::hover());
        node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, item.label));
        part(ui, &node, item.row.unwrap_or(parts::ROW));

        let mut keycap = match self.keys[i].take() {
            Some((m, a, k)) if m == mode && a == stored => k,
            _ => Keycap::new(KeycapProps {
                mode: mode.into(),
                binding: "default".into(),
                accel: stored.clone(),
                reserved: RESERVED.into(),
                prompt: "Type shortcut…".into(),
                unboundLabel: "None".into(),
                actionName: String::new(),
                disabled: false,
                ..Default::default()
            }),
        };
        let key_part = item.key.unwrap_or(parts::KEYCAP);
        let mut reset_clicked = false;
        let mut keycap_events = Vec::new();
        // A fixed-height strip: the title at the left, then the keycap and its reset at the right. Each control has
        // an id scope of its own, so the controls of different rows never share an id.
        ui.allocate_ui_with_layout(vec2(ui.available_width(), ROW_H), Layout::left_to_right(Align::Center), |ui| {
            ui.add_space(ROW_PAD_L);
            let _ = Text::new(TextProps { content: item.label.to_owned(), ..Default::default() }).ui(ui, look);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(ROW_PAD_R);
                ui.push_id(("settings-keycap", i), |ui| {
                    if show_reset {
                        let props = ButtonProps {
                            iconSize: 15 as _,
                            ..icon_button_props("restart_alt", "sm", "muted", &format!("Reset {} to default", item.label), "Reset to default")
                        };
                        reset_clicked = icon_button(ui, look, parts::RESET, props);
                        ui.add_space(rem(0.25));
                    }
                    keycap_events = as_part(ui, &[(keycap_parts::KEYCAP, key_part)], |ui| keycap.ui(ui, look));
                });
            });
        });
        self.keys[i] = Some((mode.to_owned(), stored, keycap));

        for event in keycap_events {
            match event {
                KeycapEvent::StartRecording {} => actions.push(Action::Start(i)),
                KeycapEvent::Capture { accel } => actions.push(Action::Capture(i, accel)),
                KeycapEvent::Refused { accel } => actions.push(Action::Refused(i, accel)),
                KeycapEvent::Cancel {} => actions.push(Action::Cancel(i)),
                _ => {}
            }
        }
        if reset_clicked {
            actions.push(Action::Reset(i));
        }
        if !self.errors[i].is_empty() {
            let message = self.errors[i].clone();
            error_line(ui, look, &message);
        }
    }

    fn apply(&mut self, app: &mut App, action: Action, events: &mut Vec<SettingsshortcutsEvent>) {
        match action {
            Action::Start(i) => {
                self.recording = Some(i);
                self.errors[i].clear();
            }
            Action::Cancel(i) => {
                if self.recording == Some(i) {
                    self.recording = None;
                }
            }
            Action::Capture(i, combo) => {
                if combo.is_empty() {
                    self.recording = None;
                    self.commit(app, i, String::new(), events);
                } else {
                    let accel = canonical(&combo);
                    if has_modifier(&accel) {
                        self.recording = None;
                        self.commit(app, i, accel, events);
                    } else {
                        // A bare key is not a shortcut: the row keeps recording.
                        self.errors[i] = bare_message().to_owned();
                    }
                }
            }
            Action::Refused(i, combo) => {
                self.errors[i] = format!("{} is reserved by the system.", words(&canonical(&combo)));
            }
            Action::Reset(i) => {
                let default = MAIN_ROWS[i].accel.to_owned();
                self.commit(app, i, default, events);
            }
            Action::Discard(rows) => {
                for error in self.errors.iter_mut() {
                    error.clear();
                }
                for i in rows {
                    let opened = self.opened[i].clone();
                    self.commit(app, i, opened, events);
                }
            }
        }
    }

    /// Shows the new accel at once, saves it, and reports it. A failed save puts the old accel back.
    fn commit(&mut self, app: &mut App, i: usize, accel: String, events: &mut Vec<SettingsshortcutsEvent>) {
        let previous = std::mem::replace(&mut self.accels[i], accel.clone());
        self.errors[i].clear();
        match app.data.set_setting(&setting_key(MAIN_ROWS[i].command), &accel) {
            Ok(()) => events.push(SettingsshortcutsEvent::BindingChanged { label: MAIN_ROWS[i].label.to_owned(), accel }),
            Err(e) => {
                self.accels[i] = previous;
                self.errors[i] = "Could not save this shortcut.".to_owned();
                app.refused(e);
            }
        }
    }

    /// Reads the saved bindings (a row with none saved keeps its default), and takes them as the open-time values.
    fn load(&mut self, app: &App) {
        for (i, item) in MAIN_ROWS.iter().enumerate() {
            self.accels[i] = app.data.get_setting(&setting_key(item.command)).unwrap_or_else(|| item.accel.to_owned());
        }
        self.opened = self.accels.clone();
        self.loaded = true;
    }

    /// Chooses a section. While searching, a section with matches scopes the search to it; one without clears it.
    fn choose(&mut self, next: Pane, count: usize, searching: bool) {
        self.pane = next;
        if !searching {
            return;
        }
        if count > 0 {
            self.scoped = true;
        } else {
            self.clear_query();
        }
    }

    fn clear_query(&mut self) {
        self.query.clear();
        self.scoped = false;
    }
}

/// The part a section's tab carries.
fn tab_part(pane: Pane) -> &'static str {
    match pane {
        Pane::General => parts::TABGENERAL,
        Pane::Shortcuts => parts::TABSHORTCUTS,
        Pane::About => parts::TABABOUT,
    }
}

/// The settings key a row's binding is saved under, in the app's data.
fn setting_key(command: &str) -> String {
    format!("shortcut.{command}")
}

/// Whitespace-separated words of the query, lower-cased. Empty means no search.
fn query_tokens(query: &str) -> Vec<String> {
    query.trim().to_lowercase().split_whitespace().map(str::to_owned).collect()
}

/// A row matches when every word is in its group, title, keywords or combo text (case-insensitive). The combo
/// is searched both as stored ("Mod+N") and as spoken ("Ctrl+N").
fn row_matches(row: &Row, tokens: &[String], accel: &str) -> bool {
    let haystack = format!("{GROUP} {} shortcut hotkey key binding {accel} {}", row.label, keycap_accel(accel)).to_lowercase();
    tokens.iter().all(|t| haystack.contains(t.as_str()))
}

/// Splits "Mod+Alt+K" into its modifiers and key. A key of "+" is kept whole.
fn split_accel(accel: &str) -> (Vec<&str>, &str) {
    let cut = if accel.ends_with('+') { accel.len() - 1 } else { accel.rfind('+').map_or(0, |i| i + 1) };
    let (head, key) = accel.split_at(cut);
    let mods = head.split('+').filter(|m| !m.is_empty()).collect();
    (mods, key)
}

fn has_modifier(accel: &str) -> bool {
    !split_accel(accel).0.is_empty()
}

/// A modifier in the stored form. Mod is the platform's primary key: Command on macOS, Ctrl elsewhere.
fn stored_modifier(name: &str) -> &'static str {
    match name {
        "Alt" | "Option" => "Alt",
        "Shift" => "Shift",
        "Meta" | "Super" => {
            if MAC {
                "Mod"
            } else {
                "Meta"
            }
        }
        "Control" | "Ctrl" => {
            if MAC {
                "Ctrl"
            } else {
                "Mod"
            }
        }
        _ => "Mod",
    }
}

/// The stored form of a keycap combo: the primary modifier, then Alt, then Shift, then the key.
fn canonical(combo: &str) -> String {
    let (mods, key) = split_accel(combo);
    let mut names: Vec<&str> = mods.into_iter().map(stored_modifier).collect();
    names.sort_by_key(|n| match *n {
        "Alt" => 1,
        "Shift" => 2,
        _ => 0,
    });
    names.push(key);
    names.join("+")
}

/// A modifier as it is spoken, on this platform.
fn spoken_modifier(name: &str) -> &'static str {
    match name {
        "Mod" => {
            if MAC {
                "Command"
            } else {
                "Ctrl"
            }
        }
        "Ctrl" => {
            if MAC {
                "Control"
            } else {
                "Ctrl"
            }
        }
        "Alt" => {
            if MAC {
                "Option"
            } else {
                "Alt"
            }
        }
        "Shift" => "Shift",
        "Meta" => "Super",
        _ => "Ctrl",
    }
}

/// A key as it is spoken.
fn spoken_key(key: &str) -> &str {
    match key {
        "ArrowLeft" => "Left arrow",
        "ArrowRight" => "Right arrow",
        "ArrowUp" => "Up arrow",
        "ArrowDown" => "Down arrow",
        "Enter" => "Return",
        " " => "Space",
        other => other,
    }
}

/// The keycap's accel for a stored combo: its spoken words joined with "+", so the keycap names it as it is spoken
/// ("Ctrl Alt Left arrow", "Command Option Left arrow") and draws its hint from the same words.
fn keycap_accel(stored: &str) -> String {
    if stored.is_empty() {
        return String::new();
    }
    let (mods, key) = split_accel(stored);
    let mut names: Vec<&str> = mods.into_iter().map(spoken_modifier).collect();
    names.push(spoken_key(key));
    names.join("+")
}

/// The spoken words of a stored combo: "Ctrl Shift Z", "Alt Left arrow".
fn words(stored: &str) -> String {
    keycap_accel(stored).replace('+', " ")
}

fn bare_message() -> &'static str {
    if MAC {
        "A shortcut needs Command, Option or Shift."
    } else {
        "A shortcut needs Ctrl, Alt or Shift."
    }
}

/// The defaults of an icon button: a ghost square that is in the Tab order, with its tooltip.
fn icon_button_props(icon: &str, size: &str, tone: &str, label: &str, description: &str) -> ButtonProps {
    ButtonProps {
        shape: "icon".into(),
        size: size.into(),
        variant: "ghost".into(),
        tone: tone.into(),
        icon: icon.into(),
        ariaLabel: label.into(),
        description: description.into(),
        hover: "wash".into(),
        tabbable: true,
        ..Default::default()
    }
}

/// Draws an icon button under a part of the screen. True when it was pressed.
fn icon_button(ui: &mut egui::Ui, look: Look, part_id: &str, props: ButtonProps) -> bool {
    let mut button = Button::new(props);
    let events = as_part(ui, &[(button_parts::BUTTON, part_id)], |ui| button.ui(ui, look));
    events.iter().any(|e| matches!(e, ButtonEvent::Activated { .. }))
}

/// A paragraph of Inter at a px size and weight, wrapped to the width it is given, as one labelled part.
fn paragraph(ui: &mut egui::Ui, text: &str, px: f32, w: u16, color: Color32, part_id: &str) {
    let galley = ui.painter().layout(text.to_owned(), FontId::new(px, weight(w)), color, ui.available_width().max(0.0));
    let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
    ui.painter().galley(rect.min, galley, color);
    response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, text));
    part(ui, &response, part_id);
}

/// The Main window group's heading: small caps in the muted colour. Its text is the group name as written.
fn group_label(ui: &mut egui::Ui, look: Look, text: &str) {
    let color = look.t.content_3;
    let galley = taskscape_ui::eyebrow(ui.painter(), text, color);
    let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
    ui.painter().galley(rect.min, galley, color);
    ui.ctx().accesskit_node_builder(response.id, |node| node.set_level(2));
    response.widget_info(|| WidgetInfo::labeled(Role::Heading, true, text));
    part(ui, &response, parts::GROUPMAIN);
}

/// The red line under a row whose change was refused or failed.
fn error_line(ui: &mut egui::Ui, look: Look, message: &str) {
    let color = look.t.danger_500;
    let galley = ui.painter().layout(message.to_owned(), FontId::new(rem(0.786), weight(400)), color, (ui.available_width() - ROW_PAD_L).max(0.0));
    let height = galley.size().y + rem(0.286);
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    ui.painter().galley(rect.min + vec2(ROW_PAD_L, 0.0), galley, color);
    response.widget_info(|| WidgetInfo::labeled(Role::Status, true, message));
    part(ui, &response, parts::SHORTCUTERROR);
}

/// The test's mount: the app on the props' data set, with the Settings window open on its Shortcuts page.
struct Mount {
    app: App,
    screen: SettingsShortcuts,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }
}

pub fn mount(app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let _: SettingsshortcutsProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    Ok(Box::new(Mount { app, screen: SettingsShortcuts::new() }))
}
