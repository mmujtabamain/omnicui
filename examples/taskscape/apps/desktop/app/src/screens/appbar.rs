//! contract: omnicui/contracts/appbar.contract.yaml (id: app.appbar). The title bar of the main window: the project switcher and its menu, the list tabs (inline rename, their menu, New list), and the toolbar (Swap panes, Split view, preview, Trash, Settings).
#![allow(deprecated)] // egui::Id::new, as the library widgets use it for their stable ids.
use crate::app::Sheet;
use crate::data::Id;
use crate::state::Aside;
use crate::App;
use egui::scroll_area::ScrollBarVisibility;
use egui::text::{LayoutJob, TextFormat};
use egui::{pos2, vec2, Align, Align2, Area, Color32, CornerRadius, FontFamily, FontId, Frame, Galley, Key, Layout, Margin, Order, Painter, Pos2, Rect, Role, ScrollArea, Sense, Stroke, StrokeKind, UiBuilder, WidgetInfo};
use std::collections::HashMap;
use std::sync::Arc;
use taskscape_ui::omnicui::appbar::{parts, AppbarEvent, AppbarProps};
use taskscape_ui::omnicui::button::{parts as button_parts, ButtonEvent, ButtonProps};
use taskscape_ui::omnicui::input::{parts as input_parts, InputEvent, InputProps};
use taskscape_ui::omnicui::separator::SeparatorProps;
use taskscape_ui::omnicui::statusindicator::{parts as status_parts, StatusindicatorProps};
use taskscape_ui::widgets::button::Button;
use taskscape_ui::widgets::input::Input;
use taskscape_ui::widgets::separator::Separator;
use taskscape_ui::widgets::statusindicator::StatusIndicator;
use taskscape_ui::{as_part, icon, part, weight, Look, Mounted};

// The web's sizes, in px (rem = 14 px). The bar is 46 px with its bottom rule; the content is 45 px.
const BAR_H: f32 = 46.0;
const H: f32 = 45.0;
/// The project pill's container padding: pl-[0.857rem], pr-[0.929rem].
const PILL_L: f32 = 12.0;
const PILL_R: f32 = 13.0;
/// The toolbar's padding: pl-[0.429rem], pr-[0.643rem]; gap-[0.357rem] between its items.
const TOOL_L: f32 = 6.0;
const TOOL_R: f32 = 9.0;
const GAP: f32 = 5.0;
/// An icon button at size xl (28 px), and the reduced-motion status cell (32 px).
const BTN: f32 = 28.0;
const STATUS: f32 = 32.0;
/// A tab: pl-[0.929rem], pr-[0.357rem]; its close x is 21 px, with pr-[0.5rem] after it. Max 160 px; the name 140 px.
const TAB_L: f32 = 13.0;
const TAB_R: f32 = 5.0;
const CLOSE: f32 = 21.0;
const CLOSE_PAD: f32 = 7.0;
const TAB_MAX: f32 = 160.0;
const NAME_MAX: f32 = 140.0;
const MARKER: f32 = 14.0;
/// The inline rename field: w-[9.5rem], h-[1.75rem].
const RENAME_W: f32 = 133.0;
const RENAME_H: f32 = 24.5;
/// New list: a md icon button (24.5 px) with mx-1.
const NEW_LIST: f32 = 24.5;
/// The menus: the project menu is 15 rem wide, the tab menu 11.143 rem; rows 2.286 rem and 2.143 rem tall.
const MENU_W: f32 = 210.0;
const TAB_MENU_W: f32 = 156.0;
const MENU_PAD: f32 = 3.0;
const MENU_GAP: f32 = 8.0;
const ROW_H: f32 = 32.0;
const ITEM_H: f32 = 30.0;
const ROW_PAD_L: f32 = 9.0;
const ROW_PAD_R: f32 = 5.0;
const CHECK: f32 = 15.0;
const ROW_TEXT: f32 = 13.5;
const ITEM_TEXT: f32 = 13.0;
/// Two presses on one tab within this many seconds are a double-click (the rename).
const DOUBLE_SECS: f64 = 0.5;
const NUMS: [Key; 9] = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9];

/// What a frame's clicks and chords ask for. They are applied after the frame is drawn.
enum Act {
    Project(Id),
    ProjectRename(Id),
    ProjectDelete(Id),
    ProjectNew,
    List(Id),
    ListNew,
    ListRename(Id),
    ListDelete(Id),
    ListExport(Id),
    /// A dragged tab dropped at this place among the tabs (0 = before the first).
    ListMove(Id, usize),
    /// Split view on or off; the list opened in the right pane when it turns on.
    Split(Option<Id>),
    Swap,
    Preview,
    Trash,
    Settings,
    /// An inline rename committed with this text.
    Commit(Id, String),
}

/// The bar's data for one frame: the projects, the active project's lists in tab order, and the panes.
struct Snap {
    projects: Vec<(Id, String)>,
    project: Option<usize>,
    lists: Vec<(Id, String)>,
    left: Option<usize>,
    right: Option<usize>,
    split: bool,
    active_pane: usize,
    preview: bool,
    reduced: bool,
}

impl Snap {
    fn new(app: &App) -> Self {
        let projects: Vec<(Id, String)> = app.data.projects.iter().map(|p| (p.id.clone(), p.name.clone())).collect();
        let project = app.ui.project.as_ref().and_then(|id| projects.iter().position(|(p, _)| p == id)).or((!projects.is_empty()).then_some(0));
        let lists = project.map(|i| sorted_lists(app, &projects[i].0)).unwrap_or_default();
        let find = |id: Option<&Id>| id.and_then(|id| lists.iter().position(|(l, _)| l == id));
        let left = find(app.ui.panes.first().and_then(|p| p.as_ref())).or((!lists.is_empty()).then_some(0));
        let split = app.ui.panes.len() > 1;
        let right = if split { find(app.ui.panes.get(1).and_then(|p| p.as_ref())).or(left) } else { None };
        Self {
            projects,
            project,
            lists,
            left,
            right,
            split,
            active_pane: app.ui.active_pane,
            preview: app.ui.aside == Aside::Preview,
            reduced: app.data.settings.reduced_motion == "on",
        }
    }

    fn project_id(&self) -> Option<&Id> {
        self.project.map(|i| &self.projects[i].0)
    }

    fn project_name(&self, id: &str) -> String {
        self.projects.iter().find(|(p, _)| p == id).map(|(_, n)| n.clone()).unwrap_or_default()
    }

    fn list_name(&self, id: &str) -> Option<String> {
        self.lists.iter().find(|(l, _)| l == id).map(|(_, n)| n.clone())
    }

    /// The tab whose items the keys and the highlight follow: the right one when it is the pane last clicked.
    fn focused(&self) -> Option<usize> {
        if self.split && self.active_pane == 1 && self.right.is_some() {
            self.right
        } else {
            self.left
        }
    }

    /// The first pane marker in the strip; only it carries the marker part.
    fn marker(&self) -> Option<usize> {
        if self.split {
            [self.left, self.right].into_iter().flatten().min()
        } else {
            None
        }
    }

    fn pane_side(&self, i: usize) -> Option<&'static str> {
        if !self.split {
            None
        } else if self.left == Some(i) {
            Some("left")
        } else if self.right == Some(i) {
            Some("right")
        } else {
            None
        }
    }
}

/// The active project's lists in tab order (sort_order, then creation).
fn sorted_lists(app: &App, project: &str) -> Vec<(Id, String)> {
    let mut lists = app.data.lists_of(project);
    lists.sort_by(|a, b| a.sort_order.total_cmp(&b.sort_order).then(a.created_at.cmp(&b.created_at)));
    lists.into_iter().map(|l| (l.id.clone(), l.name.clone())).collect()
}

/// An inline rename in progress: the tab's Input, drawn in place of the tab.
struct Renaming {
    list: Id,
    input: Input,
}

#[derive(Clone)]
struct TabMenu {
    list: Id,
    anchor: Pos2,
}

/// The title bar. It owns what the bar itself shows (the open project menu, the tab menu, an inline rename,
/// the status indicator); the data and the panes are the app's.
pub struct AppBar {
    project_open: bool,
    project_rect: Option<egui::Rect>,
    pill: Option<egui::Rect>,
    tab_menu: Option<TabMenu>,
    tab_menu_rect: Option<egui::Rect>,
    rename: Option<Renaming>,
    last_press: Option<(Id, f64)>,
    /// The test role of each list's tab, by list id, taken when the list first shows under its name.
    roles: HashMap<Id, &'static str>,
    /// Each list tab's box in the last frame, for a window to drop dragged tasks on.
    pub tab_rects: Vec<(Id, Rect)>,
    /// The tab dragged tasks would drop on (the window sets it while a task drag is over a tab): it lights up.
    pub drop_tab: Option<Id>,
    /// The tab being dragged to a new place, and where the pointer took it from the tab's top-left.
    tab_drag: Option<(Id, egui::Vec2)>,
    status: StatusIndicator,
}

impl AppBar {
    pub fn new() -> Self {
        Self {
            project_open: false,
            project_rect: None,
            pill: None,
            tab_menu: None,
            tab_menu_rect: None,
            rename: None,
            last_press: None,
            roles: HashMap::new(),
            tab_rects: Vec::new(),
            drop_tab: None,
            tab_drag: None,
            status: StatusIndicator::new(StatusindicatorProps {
                icon: "motion_photos_off".into(),
                label: "Reduced motion is on".into(),
                tooltip: "Reduced motion is on".into(),
                active: true,
            }),
        }
    }

    /// One frame: draws the bar and its open menus on the app, and returns the events the user's actions raised.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<AppbarEvent> {
        let snap = Snap::new(app);
        self.assign_roles(&snap);
        self.tab_rects.clear();
        let ctx = ui.ctx().clone();
        self.dismiss(ui);
        let mut acts: Vec<Act> = vec![];
        chords(ui, &snap, self.rename.is_some(), &mut acts);
        let t = look.t;

        let (bar, _) = ui.allocate_exact_size(vec2(ui.available_width(), BAR_H), Sense::hover());
        let painter = ui.painter().clone();
        painter.rect_filled(bar, CornerRadius::ZERO, t.surface_1);
        painter.hline(bar.x_range(), bar.bottom() - 0.5, Stroke::new(1.0, t.edge_2));
        // In a real window the bar is the title bar: room for the window buttons, and a drag region under
        // the controls (see chrome).
        let (lead, trail) = (crate::chrome::lead(app.native_window), crate::chrome::trail(app.native_window, 3));
        if app.native_window {
            crate::chrome::drag_region(ui, bar, true);
            if trail > 0.0 && crate::chrome::buttons(ui, look, Rect::from_min_max(pos2(bar.right() - trail, bar.top()), bar.max), true) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        let content = Rect::from_min_max(pos2(bar.min.x + lead, bar.min.y), pos2(bar.max.x - trail, bar.bottom() - 1.0));

        // The project pill: the Button widget, with the bar's own node for the switcher (its name and its value).
        let label = snap.project.map(|i| snap.projects[i].1.clone()).unwrap_or_default();
        let pill_area = Rect::from_min_max(pos2(content.left() + PILL_L, content.top()), content.max);
        let (_, pill) = at(ui, pill_area, |ui| {
            Button::new(pill_props(&label)).ui(ui, look);
        });
        let switcher = ui.interact(pill, egui::Id::new("app.appbar.projectSwitcher"), Sense::click());
        switcher.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Switch project"));
        ui.ctx().accesskit_node_builder(switcher.id, |n| n.set_value(label.clone()));
        part(ui, &switcher, parts::PROJECTSWITCHER);
        if switcher.has_focus() {
            painter.rect_stroke(pill.expand(2.0), CornerRadius::same(9), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        // A press on the pill opens the menu; a press on the pill while it is open keeps it open, as the web does.
        if switcher.clicked() {
            self.project_open = true;
            self.tab_menu = None;
        }
        self.pill = Some(pill);

        // The strip of list tabs, then the toolbar at the right.
        let right_w = toolbar_width(&snap);
        let strip_x0 = pill.right() + PILL_R;
        let strip_x1 = (content.right() - right_w).max(strip_x0);
        let strip = Rect::from_min_max(pos2(strip_x0, content.top()), pos2(strip_x1, content.bottom()));
        self.strip(ui, look, &snap, strip, &mut acts);
        self.toolbar(ui, look, &snap, content, &mut acts);
        // The bar's controls take their presses: dragging one never moves the window (#12480).
        if app.native_window {
            crate::chrome::no_drag(ui, bar, pill);
            for (_, r) in &self.tab_rects {
                crate::chrome::no_drag(ui, bar, *r);
            }
            if let Some((_, last)) = self.tab_rects.last() {
                crate::chrome::no_drag(ui, bar, Rect::from_min_max(pos2(last.right(), bar.top()), pos2(last.right() + NEW_LIST + 8.0, bar.bottom())));
            }
            crate::chrome::no_drag(ui, bar, Rect::from_min_max(pos2(content.right() - right_w, bar.top()), pos2(content.right(), bar.bottom())));
        }

        // The menus, drawn on top of the bar.
        if self.project_open {
            let (rect, a) = project_menu(&ctx, look, &snap, pos2(pill.left(), pill.bottom() + MENU_GAP));
            self.project_rect = Some(rect);
            acts.extend(a);
        } else {
            self.project_rect = None;
        }
        self.tab_menu_rect = None;
        if let Some(tm) = self.tab_menu.clone() {
            match tab_menu(&ctx, look, &snap, &tm) {
                Some((rect, a)) => {
                    self.tab_menu_rect = Some(rect);
                    acts.extend(a);
                }
                None => self.tab_menu = None,
            }
        }

        let mut ev = vec![];
        for act in acts {
            self.apply(app, &snap, act, &mut ev);
        }
        ev
    }

    /// Closes the menus on Escape, and on a press outside them (the pill is not outside: it opens the project menu).
    fn dismiss(&mut self, ui: &egui::Ui) {
        let (press, escape) = ui.input(|i| (if i.pointer.any_pressed() { i.pointer.latest_pos() } else { None }, i.key_pressed(Key::Escape)));
        if escape {
            self.project_open = false;
            self.tab_menu = None;
        }
        let Some(p) = press else { return };
        if self.project_open && !self.project_rect.is_some_and(|r| r.contains(p)) && !self.pill.is_some_and(|r| r.contains(p)) {
            self.project_open = false;
        }
        if self.tab_menu.is_some() && !self.tab_menu_rect.is_some_and(|r| r.contains(p)) {
            self.tab_menu = None;
        }
    }

    fn assign_roles(&mut self, snap: &Snap) {
        for (id, name) in &snap.lists {
            let role = match name.as_str() {
                "Reading" => "Reading",
                "Garden" => "Garden",
                _ => continue,
            };
            if !self.roles.contains_key(id) && !self.roles.values().any(|r| *r == role) {
                self.roles.insert(id.clone(), role);
            }
        }
    }

    /// The toolbar at the right: reduced-motion status (when on), Swap panes (when split), Split view, preview, Trash, Settings.
    fn toolbar(&mut self, ui: &mut egui::Ui, look: Look, snap: &Snap, content: Rect, acts: &mut Vec<Act>) {
        let cy = content.center().y;
        let mut x = content.right() - toolbar_width(snap) + TOOL_L;
        if snap.reduced {
            let r = slot(&mut x, cy, STATUS, STATUS);
            at(ui, r, |ui| {
                as_part(ui, &[(status_parts::CELL, parts::REDUCEDMOTIONSTATUS)], |ui| {
                    self.status.ui(ui, look);
                });
            });
        }
        if snap.split {
            let r = slot(&mut x, cy, BTN, BTN);
            if button(ui, look, r, tool("swap_horiz", "Swap panes", "", false, false), Some(parts::SWAPPANES)) {
                acts.push(Act::Swap);
            }
        }
        let split_label = if snap.split { "Close split view" } else { "Split view" };
        let r = slot(&mut x, cy, BTN, BTN);
        if button(ui, look, r, tool("vertical_split", split_label, "", snap.split, snap.lists.len() < 2 && !snap.split), Some(parts::SPLITVIEW)) {
            acts.push(Act::Split(None));
        }
        let (preview_label, preview_icon) = if snap.preview {
            ("Hide preview panel", "right_panel_close")
        } else {
            ("Show preview panel", "right_panel_open")
        };
        let r = slot(&mut x, cy, BTN, BTN);
        if button(ui, look, r, tool(preview_icon, preview_label, "Mod+\\", snap.preview, false), Some(parts::PREVIEWTOGGLE)) {
            acts.push(Act::Preview);
        }
        let r = slot(&mut x, cy, BTN, BTN);
        if button(ui, look, r, tool("delete", "Trash", "", false, false), Some(parts::TRASHBUTTON)) {
            acts.push(Act::Trash);
        }
        let r = slot(&mut x, cy, BTN, BTN);
        if button(ui, look, r, tool("settings", "Settings", "Mod+,", false, false), Some(parts::SETTINGSBUTTON)) {
            acts.push(Act::Settings);
        }
    }

    /// The strip of tabs for the active project: the separators, the tabs (or the inline field), and New list.
    fn strip(&mut self, ui: &mut egui::Ui, look: Look, snap: &Snap, rect: Rect, acts: &mut Vec<Act>) {
        let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)));
        let container = child.interact(rect, egui::Id::new("app.appbar.listTabs"), Sense::hover());
        container.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Lists"));
        part(&child, &container, parts::LISTTABS);
        ScrollArea::horizontal()
            .id_salt("app.appbar.lists")
            .auto_shrink([false, false])
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            // A drag on a tab moves the tab, not the strip.
            .scroll_source(egui::scroll_area::ScrollSource { drag: egui::scroll_area::DragScroll::Never, ..Default::default() })
            .show(&mut child, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| self.tabs(ui, look, snap, acts));
            });
    }

    fn tabs(&mut self, ui: &mut egui::Ui, look: Look, snap: &Snap, acts: &mut Vec<Act>) {
        // Tabs and separators sit edge to edge (egui would put 8 px between them: the gap strip, #12344).
        ui.spacing_mut().item_spacing.x = 0.0;
        let focused = snap.focused();
        let marker = snap.marker();
        let n = snap.lists.len();
        for (i, (id, _)) in snap.lists.iter().enumerate() {
            let adjacent = focused == Some(i) || focused.is_some_and(|f| f + 1 == i);
            sep(ui, look, adjacent);
            if self.rename.as_ref().is_some_and(|r| &r.list == id) {
                self.field(ui, look, acts);
            } else {
                self.tab(ui, look, snap, i, focused == Some(i), marker, acts);
            }
        }
        if n > 0 {
            sep(ui, look, focused == Some(n - 1));
        }
        self.tab_drag_update(ui, look, snap, acts);
        let (slot_rect, _) = ui.allocate_exact_size(vec2(NEW_LIST + 8.0, H), Sense::hover());
        let new_list = ButtonProps {
            variant: "ghost".into(),
            size: "md".into(),
            shape: "icon".into(),
            tone: "muted".into(),
            hover: "wash".into(),
            icon: "add".into(),
            iconSize: 18,
            ariaLabel: "New list".into(),
            description: "New list".into(),
            ..Default::default()
        };
        if button(ui, look, Rect::from_center_size(slot_rect.center(), vec2(NEW_LIST, NEW_LIST)), new_list, Some(parts::NEWLIST)) {
            acts.push(Act::ListNew);
        }
    }

    /// A tab being dragged: its ghost follows the pointer, a blue marker shows where it would land, and
    /// the release moves the list there. The other tabs keep their places until the drop (#12362).
    fn tab_drag_update(&mut self, ui: &mut egui::Ui, look: Look, snap: &Snap, acts: &mut Vec<Act>) {
        let Some((id, grab)) = self.tab_drag.clone() else { return };
        let t = look.t;
        let (pointer, released, down) = ui.input(|i| (i.pointer.interact_pos(), i.pointer.primary_released(), i.pointer.primary_down()));
        let Some(p) = pointer else { return };
        let Some(from) = snap.lists.iter().position(|(l, _)| *l == id) else {
            self.tab_drag = None;
            return;
        };
        // The place: before the first tab whose middle is right of the pointer.
        let rects: Vec<Rect> = self.tab_rects.iter().map(|(_, r)| *r).collect();
        let to = rects.iter().position(|r| p.x < r.center().x).unwrap_or(rects.len());
        let moves = to != from && to != from + 1;
        let fg = ui.ctx().layer_painter(egui::LayerId::new(Order::Tooltip, egui::Id::new("app.appbar.tabDrag")));
        let name = &snap.lists[from].1;
        let text = fg.layout_no_wrap(name.clone(), FontId::new(13.0, weight(500)), t.content_1);
        let ghost = Rect::from_min_size(p - grab, vec2(TAB_L + text.size().x + TAB_R, H));
        fg.rect_filled(ghost, CornerRadius::same(6), t.surface_3.gamma_multiply(0.85));
        fg.galley(pos2(ghost.left() + TAB_L, ghost.center().y - text.size().y / 2.0), text, t.content_1);
        // The marker over the ghost, so the place stays visible under it.
        if moves {
            let x = if to < rects.len() { rects[to].left() } else { rects.last().map(|r| r.right()).unwrap_or(p.x) };
            let mid = rects.first().map(|r| r.center().y).unwrap_or(p.y);
            fg.rect_filled(Rect::from_center_size(pos2(x, mid), vec2(2.0, 16.0)), CornerRadius::same(1), t.accent_500);
        }
        ui.ctx().request_repaint();
        if released || !down {
            self.tab_drag = None;
            if moves {
                acts.push(Act::ListMove(id, to));
            }
        }
    }

    /// The inline rename field, in the tab's place. Enter and leaving the field commit; Escape cancels.
    fn field(&mut self, ui: &mut egui::Ui, look: Look, acts: &mut Vec<Act>) {
        let (slot_rect, _) = ui.allocate_exact_size(vec2(RENAME_W, H), Sense::hover());
        let field = Rect::from_center_size(slot_rect.center(), vec2(RENAME_W, RENAME_H));
        let events = match self.rename.as_mut() {
            Some(r) => at(ui, field, |ui| as_part(ui, &[(input_parts::FIELD, parts::LISTRENAMEINPUT)], |ui| r.input.ui(ui, look))).0,
            None => return,
        };
        let Some(list) = self.rename.as_ref().map(|r| r.list.clone()) else { return };
        for e in events {
            match e {
                InputEvent::Commit { value } => {
                    self.rename = None;
                    acts.push(Act::Commit(list.clone(), value));
                }
                InputEvent::Cancel {} => self.rename = None,
                _ => {}
            }
        }
    }

    /// One tab: its highlight, its name (a button named by the list), its pane marker and its close x.
    #[allow(clippy::too_many_arguments)]
    fn tab(&mut self, ui: &mut egui::Ui, look: Look, snap: &Snap, i: usize, selected: bool, marker: Option<usize>, acts: &mut Vec<Act>) {
        let t = look.t;
        let (id, name) = &snap.lists[i];
        let side = snap.pane_side(i);
        let marker_w = if side.is_some() { MARKER + GAP } else { 0.0 };
        let font = FontId::new(13.0, weight(500));
        let color = if selected { t.content_1 } else { t.content_2 };
        let text_max = (TAB_MAX - TAB_L - TAB_R - CLOSE - CLOSE_PAD - marker_w).min(NAME_MAX);
        let text = label(ui.painter(), name, font, color, text_max);
        let tab_w = TAB_L + marker_w + text.size().x + TAB_R;
        let wrap_w = tab_w + CLOSE + CLOSE_PAD;
        let (wrap, _) = ui.allocate_exact_size(vec2(wrap_w, H), Sense::hover());
        self.tab_rects.push((id.clone(), wrap));
        let painter = ui.painter().clone();
        let hovered = ui.rect_contains_pointer(wrap) && self.tab_drag.is_none();
        let dragged = self.tab_drag.as_ref().is_some_and(|(d, _)| d == id);
        // A tab taking dropped tasks lights up as the reference's does (#12361).
        if self.drop_tab.as_ref() == Some(id) {
            painter.rect_filled(wrap, CornerRadius::ZERO, t.wash_2);
        } else if selected {
            painter.rect_filled(wrap, CornerRadius::ZERO, t.surface_2);
        } else if hovered {
            painter.rect_filled(wrap, CornerRadius::ZERO, t.wash_1);
        }

        let tab = Rect::from_min_size(wrap.min, vec2(tab_w, H));
        let r = ui.interact(tab, egui::Id::new(("app.appbar.tab", id.clone())), Sense::click_and_drag());
        if r.drag_started() {
            let grab = ui.input(|i| i.pointer.press_origin()).map(|p| p - wrap.min).unwrap_or_default();
            self.tab_drag = Some((id.clone(), grab));
        }
        r.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, name.clone()));
        ui.ctx().accesskit_node_builder(r.id, |n| n.set_selected(selected));
        if let Some(p) = tab_part(i, self.roles.get(id).copied().or((name == "Hardware").then_some("Hardware"))) {
            part(ui, &r, p);
        }

        // The pane marker: a decorative icon in the tab, one node for the first marker of the strip.
        if let Some(side) = side {
            let c = pos2(tab.left() + TAB_L + MARKER / 2.0, tab.center().y);
            let glyph = if side == "left" { "line_start_circle" } else { "line_end_circle" };
            painter.text(c, Align2::CENTER_CENTER, icon(glyph, MARKER).text(), FontId::new(MARKER, FontFamily::Name("Icons".into())), t.accent_500);
            if marker == Some(i) {
                let m = ui.interact(Rect::from_center_size(c, vec2(MARKER, MARKER)), egui::Id::new(("app.appbar.marker", id.clone())), Sense::hover());
                let pane = if side == "left" { "Left pane" } else { "Right pane" };
                m.widget_info(|| WidgetInfo::labeled(Role::Image, true, pane));
                part(ui, &m, parts::PANEMARKER);
            }
        }
        // The dragged tab stays in its place, dimmed: the tabs do not shift while it moves (#12362).
        let color = if dragged { color.gamma_multiply(0.35) } else { color };
        painter.galley(pos2(tab.left() + TAB_L + marker_w, tab.center().y - text.size().y / 2.0), text, color);
        if r.has_focus() {
            painter.rect_stroke(tab, CornerRadius::ZERO, Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
        }

        // A press selects the list; two presses in quick succession (or a double-click) start its rename.
        if r.clicked() {
            let now = ui.input(|i| i.time);
            let twice = r.double_clicked() || self.last_press.as_ref().is_some_and(|(l, at)| l == id && now - *at < DOUBLE_SECS);
            if twice {
                self.last_press = None;
                self.begin_rename(id, name);
            } else {
                self.last_press = Some((id.clone(), now));
                acts.push(Act::List(id.clone()));
            }
        }
        if r.secondary_clicked() {
            self.open_tab_menu(id, wrap);
        }
        if r.has_focus() {
            let (menu_key, f2) = ui.input(|i| (i.key_pressed(Key::Menu) || (i.key_pressed(Key::F10) && i.modifiers.shift), i.key_pressed(Key::F2)));
            if menu_key {
                self.open_tab_menu(id, wrap);
            } else if f2 {
                self.begin_rename(id, name);
            }
        }

        // The close x: the first tab's is the screen's Delete list part.
        let close = Rect::from_min_size(pos2(tab.right(), wrap.center().y - CLOSE / 2.0), vec2(CLOSE, CLOSE));
        let close_part = match (i, name.as_str()) {
            (0, _) => Some(parts::LISTTABDELETE),
            (_, "Reading") => Some(TAB_DELETE_READING),
            _ => None,
        };
        if button(ui, look, close, icon_button("close", 15.0, "Delete list", "muted", "sm"), close_part) {
            acts.push(Act::ListDelete(id.clone()));
        }
    }

    fn begin_rename(&mut self, id: &Id, name: &str) {
        self.tab_menu = None;
        self.rename = Some(Renaming {
            list: id.clone(),
            input: Input::new(InputProps {
                value: name.to_owned(),
                ariaLabel: "List name".into(),
                mode: "draft".into(),
                size: "inline".into(),
                autoFocus: true,
                selectOnFocus: true,
                ..Default::default()
            }),
        });
    }

    fn open_tab_menu(&mut self, id: &Id, wrap: Rect) {
        self.project_open = false;
        self.tab_menu = Some(TabMenu { list: id.clone(), anchor: pos2(wrap.left(), wrap.bottom() + 4.0) });
    }

    /// Applies one action to the app, and raises its event.
    fn apply(&mut self, app: &mut App, snap: &Snap, act: Act, ev: &mut Vec<AppbarEvent>) {
        let left = snap.left.map(|i| snap.lists[i].0.clone());
        let right = snap.right.map(|i| snap.lists[i].0.clone());
        match act {
            Act::Project(id) => {
                self.project_open = false;
                if snap.project_id() == Some(&id) {
                    return;
                }
                let name = snap.project_name(&id);
                let first = sorted_lists(app, &id).first().map(|l| l.0.clone());
                app.ui.set_project(Some(id));
                app.ui.open_list(first, Some(0));
                app.remember();
                ev.push(AppbarEvent::ProjectSelected { name });
            }
            Act::ProjectRename(id) => {
                self.project_open = false;
                let name = snap.project_name(&id);
                ev.push(AppbarEvent::ProjectRenameRequested { name });
                app.sheet = Some(Sheet::RenameProject(id));
            }
            Act::ProjectDelete(id) => {
                self.project_open = false;
                let name = snap.project_name(&id);
                ev.push(AppbarEvent::ProjectDeleteRequested { name });
                app.sheet = Some(Sheet::DeleteProject(id));
            }
            Act::ProjectNew => {
                self.project_open = false;
                let active = snap.project.map(|i| snap.projects[i].1.clone()).unwrap_or_default();
                ev.push(AppbarEvent::ProjectCreateRequested { activeProject: active });
                app.sheet = Some(Sheet::NewProject);
            }
            Act::List(id) => {
                if left.as_ref() == Some(&id) {
                    return;
                }
                let name = snap.list_name(&id).unwrap_or_default();
                app.ui.open_list(Some(id), Some(0));
                app.remember();
                ev.push(AppbarEvent::ListSelected { name });
            }
            Act::ListNew => {
                if let Some(i) = snap.project {
                    let project = snap.projects[i].1.clone();
                    ev.push(AppbarEvent::ListCreateRequested { project });
                    app.sheet = Some(Sheet::NewList);
                }
            }
            Act::ListRename(id) => {
                self.tab_menu = None;
                let name = snap.list_name(&id).unwrap_or_default();
                ev.push(AppbarEvent::ListRenameRequested { name });
                app.sheet = Some(Sheet::RenameList(id));
            }
            Act::ListDelete(id) => {
                self.tab_menu = None;
                let name = snap.list_name(&id).unwrap_or_default();
                ev.push(AppbarEvent::ListDeleteRequested { name });
                app.sheet = Some(Sheet::DeleteList(id));
            }
            Act::ListMove(id, to) => {
                // Between the two lists either side of the place, by sort order.
                let Some(project) = app.ui.project.clone() else { return };
                let orders: Vec<(Id, f64)> = app.data.lists_of(&project).iter().map(|l| (l.id.clone(), l.sort_order)).collect();
                let before = to.checked_sub(1).and_then(|i| orders.get(i)).map(|(_, o)| *o);
                let after = orders.get(to).map(|(_, o)| *o);
                let order = match (before, after) {
                    (Some(a), Some(b)) => (a + b) / 2.0,
                    (Some(a), None) => a + 1.0,
                    (None, Some(b)) => b - 1.0,
                    (None, None) => return,
                };
                if let Err(e) = app.data.reorder_list(&id, order) {
                    app.refused(e);
                }
            }
            Act::ListExport(id) => {
                self.tab_menu = None;
                let name = snap.list_name(&id).unwrap_or_default();
                ev.push(AppbarEvent::ListExportRequested { name });
                app.export_list(&id);
            }
            Act::Split(target) => {
                self.tab_menu = None;
                let Some(left) = left else { return };
                if snap.split {
                    app.ui.panes = vec![Some(left)];
                    app.ui.active_pane = 0;
                    ev.push(AppbarEvent::SplitToggled { active: false });
                } else {
                    let right = target.unwrap_or_else(|| left.clone());
                    app.ui.panes = vec![Some(left), Some(right)];
                    app.ui.active_pane = 0;
                    ev.push(AppbarEvent::SplitToggled { active: true });
                }
            }
            Act::Swap => {
                let (Some(l), Some(r)) = (left, right) else { return };
                let name = snap.list_name(&r).unwrap_or_default();
                app.ui.panes = vec![Some(r), Some(l)];
                app.ui.active_pane = 0;
                ev.push(AppbarEvent::PanesSwapped { left: name });
            }
            Act::Preview => {
                let open = !snap.preview;
                app.ui.aside = if open { Aside::Preview } else { Aside::None };
                ev.push(AppbarEvent::PreviewToggled { open });
            }
            Act::Trash => {
                app.trash_window = true;
                ev.push(AppbarEvent::TrashOpened { open: true });
            }
            Act::Settings => {
                app.settings_window = Some("general".to_owned());
                ev.push(AppbarEvent::SettingsOpened { open: true });
            }
            Act::Commit(id, text) => {
                let name = text.trim().to_owned();
                if name.is_empty() || snap.list_name(&id).as_deref() == Some(name.as_str()) {
                    return;
                }
                match app.data.rename_list(&id, &name) {
                    Ok(()) => ev.push(AppbarEvent::ListRenamed { name }),
                    Err(e) => app.refused(e),
                }
            }
        }
    }
}

/// The keyboard chords of the bar (the desktop's defaults, with the platform modifier): next and previous
/// project, next and previous list, the list number, and New list. Ignored while a text field has focus.
fn chords(ui: &egui::Ui, snap: &Snap, renaming: bool, acts: &mut Vec<Act>) {
    // While an inline name is typed, the keys are the field's (egui's "wants keyboard input" is true for any focus, so it is not used).
    if renaming {
        return;
    }
    // The platform modifier: Command on macOS, Control elsewhere (egui sets `command` from the OS, so the raw flags are read too).
    let (cmd, shift, alt) = ui.input(|i| (i.modifiers.command || i.modifiers.ctrl || i.modifiers.mac_cmd, i.modifiers.shift, i.modifiers.alt));
    if !cmd {
        return;
    }
    let press = |k: Key| ui.input(|i| i.key_pressed(k));
    let np = snap.projects.len() as i64;
    let nl = snap.lists.len() as i64;
    if shift && !alt {
        if np > 0 && press(Key::CloseBracket) {
            let cur = snap.project.map_or(-1, |i| i as i64);
            acts.push(Act::Project(snap.projects[(cur + 1).rem_euclid(np) as usize].0.clone()));
        }
        if np > 0 && press(Key::OpenBracket) {
            let cur = snap.project.map_or(-1, |i| i as i64);
            acts.push(Act::Project(snap.projects[(cur - 1).rem_euclid(np) as usize].0.clone()));
        }
        if press(Key::N) {
            acts.push(Act::ListNew);
        }
    } else if alt && !shift {
        if nl > 0 && press(Key::ArrowRight) {
            let cur = snap.left.map_or(-1, |i| i as i64);
            acts.push(Act::List(snap.lists[(cur + 1).rem_euclid(nl) as usize].0.clone()));
        }
        if nl > 0 && press(Key::ArrowLeft) {
            let cur = snap.left.map_or(-1, |i| i as i64);
            acts.push(Act::List(snap.lists[(cur - 1).rem_euclid(nl) as usize].0.clone()));
        }
    } else if !alt && !shift {
        for (n, key) in NUMS.iter().enumerate() {
            if n < snap.lists.len() && press(*key) {
                acts.push(Act::List(snap.lists[n].0.clone()));
            }
        }
    }
}

/// The toolbar's width: its padding, its items and the gaps between them.
fn toolbar_width(snap: &Snap) -> f32 {
    let mut items = vec![BTN, BTN, BTN, BTN];
    if snap.split {
        items.push(BTN);
    }
    if snap.reduced {
        items.push(STATUS);
    }
    TOOL_L + items.iter().sum::<f32>() + GAP * (items.len() as f32 - 1.0) + TOOL_R
}

/// The project menu: the project rows, a divider, and New project. Returns its rect and what was pressed.
fn project_menu(ctx: &egui::Context, look: Look, snap: &Snap, anchor: Pos2) -> (Rect, Vec<Act>) {
    let t = look.t;
    let mut acts = vec![];
    let area = Area::new(egui::Id::new("app.appbar.projectMenu")).order(Order::Foreground).fixed_pos(anchor).show(ctx, |ui| {
        let frame = Frame::popup(ui.style())
            .fill(t.surface_3)
            .stroke(Stroke::new(1.0, t.edge_1))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(3));
        let f = frame.show(ui, |ui| {
            ui.set_width(MENU_W - 2.0 * MENU_PAD);
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            for (i, (id, name)) in snap.projects.iter().enumerate() {
                let parts3 = match i {
                    0 => (Some(parts::PROJECTROW), Some(parts::PROJECTRENAME), Some(parts::PROJECTDELETE)),
                    1 => (Some(parts::PROJECTROWSPRING), Some(PROJECT_RENAME_SPRING), Some(parts::PROJECTDELETESPRING)),
                    // The last row of three or more (a project a test created is appended here).
                    n if n + 1 == snap.projects.len() => (Some(PROJECT_ROW_LAST), None, None),
                    _ => (None, None, None),
                };
                project_row(ui, look, id, name, snap.project == Some(i), parts3, &mut acts);
            }
            divider(ui, look);
            new_project_row(ui, look, &mut acts);
        });
        let rect = f.response.rect;
        let node = ui.interact(rect, egui::Id::new("app.appbar.projectMenu.panel"), Sense::hover());
        node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &node, parts::PROJECTMENU);
        rect
    });
    (area.inner, acts)
}

/// One project row: the name (a menuitem, selected when it is the active project), then Rename and Delete,
/// always shown.
fn project_row(ui: &mut egui::Ui, look: Look, id: &Id, name: &str, active: bool, parts3: (Option<&str>, Option<&str>, Option<&str>), acts: &mut Vec<Act>) {
    let t = look.t;
    let (row_part, rename_part, delete_part) = parts3;
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::hover());
    let hovered = ui.rect_contains_pointer(row);
    let del = Rect::from_min_size(pos2(row.right() - ROW_PAD_R - CLOSE, row.center().y - CLOSE / 2.0), vec2(CLOSE, CLOSE));
    let ren = del.translate(vec2(-CLOSE, 0.0));
    let name_rect = Rect::from_min_max(row.min, pos2(ren.left(), row.bottom()));

    let r = ui.interact(name_rect, egui::Id::new(("app.appbar.project", id.clone())), Sense::click());
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
    ui.ctx().accesskit_node_builder(r.id, |n| n.set_selected(active));
    if let Some(p) = row_part {
        part(ui, &r, p);
    }
    let painter = ui.painter().clone();
    if hovered || r.has_focus() {
        painter.rect_filled(row, CornerRadius::same(5), t.wash_2);
    }
    if r.has_focus() {
        painter.rect_stroke(name_rect, CornerRadius::same(5), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
    }
    let mid = row.center().y;
    if active {
        painter.text(pos2(name_rect.left() + ROW_PAD_L + CHECK / 2.0, mid), Align2::CENTER_CENTER, icon("check", CHECK).text(), FontId::new(CHECK, FontFamily::Name("Icons".into())), t.accent_500);
    }
    let tx = name_rect.left() + ROW_PAD_L + CHECK + GAP;
    let text = label(&painter, name, FontId::new(ROW_TEXT, weight(400)), t.content_1, (name_rect.right() - tx - 4.0).max(0.0));
    painter.galley(pos2(tx, mid - text.size().y / 2.0), text, t.content_1);

    if button(ui, look, ren, icon_button("edit", 16.0, "Rename project", "muted", "sm"), rename_part) {
        acts.push(Act::ProjectRename(id.clone()));
    }
    if button(ui, look, del, icon_button("delete", 16.0, "Delete project", "danger", "sm"), delete_part) {
        acts.push(Act::ProjectDelete(id.clone()));
    }
    if r.clicked() {
        acts.push(Act::Project(id.clone()));
    }
}

/// New project…, the last row of the project menu.
fn new_project_row(ui: &mut egui::Ui, look: Look, acts: &mut Vec<Act>) {
    let t = look.t;
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::hover());
    let r = ui.interact(row, egui::Id::new("app.appbar.projectNew"), Sense::click());
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, "New project…"));
    part(ui, &r, parts::PROJECTNEW);
    let painter = ui.painter().clone();
    if r.hovered() || r.has_focus() {
        painter.rect_filled(row, CornerRadius::same(5), t.wash_2);
    }
    if r.has_focus() {
        painter.rect_stroke(row, CornerRadius::same(5), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
    }
    let mid = row.center().y;
    painter.text(pos2(row.left() + ROW_PAD_L + CHECK / 2.0, mid), Align2::CENTER_CENTER, icon("add", CHECK).text(), FontId::new(CHECK, FontFamily::Name("Icons".into())), t.content_2);
    painter.text(pos2(row.left() + ROW_PAD_L + CHECK + GAP, mid), Align2::LEFT_CENTER, "New project…", FontId::new(ROW_TEXT, weight(400)), t.content_1);
    if r.clicked() {
        acts.push(Act::ProjectNew);
    }
}

/// The tab menu: Rename, Open in split view (or Close split), Export list, Delete list.
fn tab_menu(ctx: &egui::Context, look: Look, snap: &Snap, tm: &TabMenu) -> Option<(Rect, Vec<Act>)> {
    let i = snap.lists.iter().position(|(id, _)| *id == tm.list)?;
    let id = snap.lists[i].0.clone();
    let t = look.t;
    let split_label = if snap.split { "Close split" } else { "Open in split view" };
    let mut acts = vec![];
    let area = Area::new(egui::Id::new("app.appbar.tabMenu")).order(Order::Foreground).fixed_pos(tm.anchor).show(ctx, |ui| {
        let frame = Frame::popup(ui.style())
            .fill(t.surface_3)
            .stroke(Stroke::new(1.0, t.edge_1))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(3));
        let f = frame.show(ui, |ui| {
            ui.set_width(TAB_MENU_W - 2.0 * MENU_PAD);
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            if menu_item(ui, look, "Rename", parts::TABMENURENAME, false) {
                acts.push(Act::ListRename(id.clone()));
            }
            if menu_item(ui, look, split_label, parts::TABMENUSPLIT, false) {
                acts.push(Act::Split(Some(id.clone())));
            }
            if menu_item(ui, look, "Export list", parts::TABMENUEXPORT, false) {
                acts.push(Act::ListExport(id.clone()));
            }
            if menu_item(ui, look, "Delete list", parts::TABMENUDELETE, true) {
                acts.push(Act::ListDelete(id.clone()));
            }
        });
        let rect = f.response.rect;
        let node = ui.interact(rect, egui::Id::new("app.appbar.tabMenu.panel"), Sense::hover());
        node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &node, parts::TABMENU);
        rect
    });
    Some((area.inner, acts))
}

/// One row of the tab menu: a button by its label; true when it was pressed.
fn menu_item(ui: &mut egui::Ui, look: Look, text: &str, part_id: &str, danger: bool) -> bool {
    let t = look.t;
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), ITEM_H), Sense::hover());
    let r = ui.interact(row, egui::Id::new(part_id), Sense::click());
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, text));
    part(ui, &r, part_id);
    let painter = ui.painter().clone();
    if r.hovered() || r.has_focus() {
        painter.rect_filled(row, CornerRadius::same(5), t.wash_2);
    }
    if r.has_focus() {
        painter.rect_stroke(row, CornerRadius::same(5), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
    }
    let color = if danger { t.danger_500 } else { t.content_1 };
    painter.text(pos2(row.left() + ROW_PAD_L, row.center().y), Align2::LEFT_CENTER, text, FontId::new(ITEM_TEXT, weight(400)), color);
    r.clicked()
}

/// The hairline between the project rows and New project, inset 8 px.
fn divider(ui: &mut egui::Ui, look: Look) {
    ui.add_space(MENU_PAD);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.allocate_ui(vec2(MENU_W - 2.0 * MENU_PAD - 16.0, 1.0), |ui| {
            Separator::new(SeparatorProps { orientation: "horizontal".into(), ..Default::default() }).ui(ui, look);
        });
    });
    ui.add_space(MENU_PAD);
}

/// A vertical hairline in the strip: short beside the tabs, full height beside the focused one.
fn sep(ui: &mut egui::Ui, look: Look, adjacent: bool) {
    Separator::new(SeparatorProps {
        orientation: "vertical".into(),
        length: (if adjacent { "full" } else { "sm" }).into(),
        emphasis: (if adjacent { "adjacent" } else { "default" }).into(),
        ..Default::default()
    })
    .ui(ui, look);
}

/// Parts no test of this contract names, for a window around the title bar to name: the Reading tab's
/// Delete list, and the tab a New list test adds (Hardware).
pub const TAB_DELETE_READING: &str = "app.appbar.listTabDeleteReading";
pub const TAB_HARDWARE: &str = "app.appbar.listTabHardware";
/// The second project's Rename, and the last project row of three or more.
pub const PROJECT_RENAME_SPRING: &str = "app.appbar.projectRenameSpring";
pub const PROJECT_ROW_LAST: &str = "app.appbar.projectRowLast";

/// The test part of a tab: the first tab is listTab; Reading and Garden have their own names.
fn tab_part(i: usize, role: Option<&'static str>) -> Option<&'static str> {
    if i == 0 {
        return Some(parts::LISTTAB);
    }
    match role {
        Some("Reading") => Some(parts::LISTTABREADING),
        Some("Garden") => Some(parts::LISTTABGARDEN),
        Some("Hardware") => Some(TAB_HARDWARE),
        _ => None,
    }
}

/// Draws a library widget in a child of `ui` over `rect`, without taking space from `ui`'s flow.
/// Returns what `add` returned and the rect the widget used.
fn at<R>(ui: &mut egui::Ui, rect: Rect, add: impl FnOnce(&mut egui::Ui) -> R) -> (R, Rect) {
    let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)));
    let out = add(&mut child);
    (out, child.min_rect())
}

/// A Button widget over `rect`, under the screen's part name when it has one. True when it was pressed.
fn button(ui: &mut egui::Ui, look: Look, rect: Rect, p: ButtonProps, part_id: Option<&str>) -> bool {
    let (events, _) = at(ui, rect, |ui| match part_id {
        Some(id) => as_part(ui, &[(button_parts::BUTTON, id)], |ui| Button::new(p).ui(ui, look)),
        None => Button::new(p).ui(ui, look),
    });
    events.iter().any(|e| matches!(e, ButtonEvent::Activated { .. }))
}

/// The project pill: a neutral xl button with the project's name and the unfold glyph. Its own focus is off:
/// the bar's switcher node takes the focus.
fn pill_props(name: &str) -> ButtonProps {
    ButtonProps {
        variant: "neutral".into(),
        size: "xl".into(),
        tone: "strong".into(),
        width: "content".into(),
        label: name.into(),
        trailingIcon: "unfold_more".into(),
        iconSize: 15,
        tabbable: false,
        ariaLabel: "Switch project".into(),
        description: "Switch project".into(),
        ..Default::default()
    }
}

/// A toolbar icon button at size xl: a ghost with the muted tone, a wash on hover.
fn tool(icon_name: &str, label_text: &str, shortcut: &str, active: bool, disabled: bool) -> ButtonProps {
    ButtonProps {
        variant: "ghost".into(),
        size: "xl".into(),
        shape: "icon".into(),
        tone: "muted".into(),
        hover: "wash".into(),
        icon: icon_name.into(),
        iconSize: 19,
        ariaLabel: label_text.into(),
        description: label_text.into(),
        shortcut: shortcut.into(),
        active,
        disabled,
        ..Default::default()
    }
}

/// A small icon button (size sm, 21 px) for a tab's close x or a project row's action.
fn icon_button(icon_name: &str, glyph: f32, label_text: &str, tone: &str, size: &str) -> ButtonProps {
    ButtonProps {
        variant: "ghost".into(),
        size: size.into(),
        shape: "icon".into(),
        tone: tone.into(),
        hover: "wash".into(),
        icon: icon_name.into(),
        iconSize: glyph as i64,
        ariaLabel: label_text.into(),
        description: label_text.into(),
        ..Default::default()
    }
}

/// Takes `w` along the row from `x`, and moves `x` past it and the gap.
fn slot(x: &mut f32, cy: f32, w: f32, h: f32) -> Rect {
    let r = Rect::from_center_size(pos2(*x + w / 2.0, cy), vec2(w, h));
    *x += w + GAP;
    r
}

/// A name laid out on one line, ellipsis when it is longer than `max_w`.
fn label(painter: &Painter, s: &str, font: FontId, color: Color32, max_w: f32) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(s.to_owned(), TextFormat { font_id: font, color, ..Default::default() });
    job.wrap.max_width = max_w;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    painter.layout_job(job)
}

/// The test's mount: the app on the props' data set, with the account's reduced-motion choice set before the first frame.
struct Mount {
    app: App,
    screen: AppBar,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }
}

pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: AppbarProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    let choice = match p.reducedMotion.as_str() {
        "on" => "on",
        "system" => "system",
        _ => "off",
    };
    app.data.set_setting("reduced_motion", choice).map_err(|e| e.to_string())?;
    Ok(Box::new(Mount { app, screen: AppBar::new() }))
}
