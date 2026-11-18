//! contract: omnicui/contracts/multiinspector.contract.yaml (id: app.multiinspector). The multi-select inspector: the body of the preview panel while two or more tasks are selected in the focused pane, with the count, the done tally and one row per selected task. No verb grid (decision 6).
use crate::data::Id;
use crate::App;
use egui::{pos2, vec2, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Key, Layout, Rect, Response, Role, Sense, Stroke, StrokeKind, UiBuilder, WidgetInfo};
use taskscape_ui::omnicui::badge::BadgeProps;
use taskscape_ui::omnicui::button::ButtonProps;
use taskscape_ui::omnicui::checkbox::CheckboxProps;
use taskscape_ui::omnicui::multiinspector::{parts, MultiinspectorEvent, MultiinspectorProps};
use taskscape_ui::widgets::badge::Badge;
use taskscape_ui::widgets::button::Button;
use taskscape_ui::widgets::checkbox::Checkbox;
use taskscape_ui::{icon, part, rem, weight, Look, Mounted};

/// The title of the parcel row: the row the contract names as parcelCheck and parcelTitle.
const PARCEL_TITLE: &str = "Post the parcel to Mum";

/// The harness selections, by title in list order (contract: selection four | two).
const FOUR: [&str; 4] = ["Pick up prescription", "Post the parcel to Mum", "Book bike service", "Cut a spare key"];
const TWO: [&str; 2] = ["Book bike service", "Cut a spare key"];

/// Sizes from the web's Tailwind classes: space-7 and space-5 and space-4, the row height h-9, the icon button.
const PAD: f32 = rem(0.875);
const GAP: f32 = rem(0.5);
const INSET: f32 = rem(0.375);
const ROW_H: f32 = rem(2.25);
const TITLE_H: f32 = rem(1.75);
const ICON: f32 = 21.0;

/// Which contract part a row carries: the first row, the parcel row, the last row, or none.
#[derive(Clone, Copy, PartialEq)]
enum Slot {
    First,
    Parcel,
    Last,
    Other,
}

/// One selected task, read from the app for this frame.
struct Row {
    id: Id,
    title: String,
    done: bool,
    active: bool,
    list: Option<String>,
    slot: Slot,
}

/// What a press does. Collected while the frame is drawn, applied after it.
enum Act {
    Check(Id, bool),
    Peek(Id),
    Open(Id),
    Clear,
}

/// The multi-select inspector. It keeps nothing between frames: the selection and the peeked task are
/// in `app.ui`, and the done flags are in `app.data`.
#[derive(Default)]
pub struct MultiInspector {}

impl MultiInspector {
    pub fn new() -> Self {
        Self {}
    }

    /// One frame, on the app: what the main window calls, and what the test mounts.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<MultiinspectorEvent> {
        let rows = selected_rows(app);
        // Fewer than two selected tasks: the single-task inspector takes over, so nothing is drawn here.
        if rows.len() < 2 {
            return vec![];
        }
        let t = look.t;
        let count = rows.len();
        let done = rows.iter().filter(|r| r.done).count();
        let mut acts: Vec<Act> = vec![];

        // The panel: its surface and its leading edge (the web's bg-surface-1 and border-l).
        let area = ui.available_rect_before_wrap();
        let root = ui.interact(area, egui::Id::unique("multiinspector.root"), Sense::hover());
        root.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &root, parts::INSPECTOR);
        ui.painter().rect_filled(area, 0.0, t.surface_1);
        ui.painter().line_segment([area.left_top(), area.left_bottom()], Stroke::new(1.0, t.edge_1));

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(GAP, 0.0);
            ui.add_space(PAD);
            header(ui, look, count, done, &mut acts);
            ui.add_space(PAD);
            selected_list(ui, look, &rows, &mut acts);
        });

        if !acts.is_empty() {
            ui.ctx().request_repaint();
        }
        apply(app, acts);
        vec![]
    }
}

/// The selected tasks of the focused pane, in list order. Tasks outside that list are not shown.
fn selected_rows(app: &App) -> Vec<Row> {
    let Some(list) = app.active_list() else {
        return vec![];
    };
    let picked: Vec<(Id, String, bool, Id)> = app
        .data
        .tasks_of(&list)
        .into_iter()
        .filter(|t| app.ui.selection.contains(&t.id))
        .map(|t| (t.id.clone(), t.title.clone(), t.done, t.list_id.clone()))
        .collect();
    let count = picked.len();
    picked
        .into_iter()
        .enumerate()
        .map(|(i, (id, title, done, list_id))| {
            let slot = if i == 0 {
                Slot::First
            } else if title == PARCEL_TITLE {
                Slot::Parcel
            } else if i + 1 == count {
                Slot::Last
            } else {
                Slot::Other
            };
            Row {
                active: app.ui.preview.as_deref() == Some(id.as_str()),
                list: app.data.list(&list_id).map(|l| l.name.clone()),
                id,
                title,
                done,
                slot,
            }
        })
        .collect()
}

/// The header: the count, the headline and the tally, and Clear selection.
fn header(ui: &mut egui::Ui, look: Look, count: usize, done: usize, acts: &mut Vec<Act>) {
    let t = look.t;
    let count_s = count.to_string();
    let inner = ui.available_width() - 2.0 * PAD;
    let col_w = (inner - 28.0 - ICON - 2.0 * GAP).max(0.0);
    ui.horizontal(|ui| {
        ui.add_space(PAD);

        // The count: the Badge widget (lg, round, accent), tagged with the contract's part.
        let badge = BadgeProps { children: count_s.clone(), tone: "accent".into(), shape: "round".into(), size: "lg".into(), ..Default::default() };
        let badge_rect = ui.scope(|ui| {
            Badge::new(badge).ui(ui, look);
        })
        .response
        .rect;
        tag(ui, egui::Id::unique("multiinspector.count"), badge_rect, Sense::hover(), Some(parts::COUNT), || value_info(&count_s, &count_s));

        ui.vertical(|ui| {
            ui.set_width(col_w);
            line(ui, &format!("{count} tasks selected"), rem(1.143), 600, t.content_1, rem(1.714), parts::HEADLINE);
            line(ui, &format!("{done} done / {count}"), rem(0.821), 400, t.content_2, rem(1.232), parts::TALLY);
        });

        // Clear selection: the Button widget (ghost, close), tagged with the contract's part.
        let clear = ButtonProps {
            variant: "ghost".into(),
            tone: "muted".into(),
            size: "sm".into(),
            shape: "icon".into(),
            width: "content".into(),
            icon: "close".into(),
            iconSize: 16,
            ariaLabel: "Clear selection".into(),
            description: "Clear selection".into(),
            ..Default::default()
        };
        let clear_rect = ui.scope(|ui| {
            Button::new(clear).ui(ui, look);
        })
        .response
        .rect;
        let clear_btn = tag(ui, egui::Id::unique("multiinspector.clear"), clear_rect, Sense::click(), Some(parts::CLEARSELECTION), || WidgetInfo::labeled(Role::Button, true, "Clear selection"));
        if clear_btn.clicked() {
            acts.push(Act::Clear);
        }
    });
}

/// The scrolling list: the "Selected" section label, then one row per selected task.
fn selected_list(ui: &mut egui::Ui, look: Look, rows: &[Row], acts: &mut Vec<Act>) {
    egui::ScrollArea::vertical().id_salt("multiinspector.selected").auto_shrink([false, false]).show(ui, |ui| {
        ui.add_space(rem(0.714));
        ui.horizontal(|ui| {
            ui.add_space(PAD);
            section_label(ui, look);
        });
        ui.add_space(rem(0.5));
        // The rows reach 5.25px into the padding on each side (the web's -mx-1.5).
        let width = (ui.available_width() - 2.0 * (PAD - INSET)).max(0.0);
        ui.horizontal(|ui| {
            ui.add_space(PAD - INSET);
            ui.vertical(|ui| {
                ui.set_width(width);
                for row in rows {
                    draw_row(ui, look, row, width, acts);
                }
            });
        });
        ui.add_space(PAD);
    });
}

/// One selected task: the done check, the title (which peeks the task), the list name, and an Open
/// button that shows while the row is hovered and narrows the selection to this task.
fn draw_row(ui: &mut egui::Ui, look: Look, row: &Row, width: f32, acts: &mut Vec<Act>) {
    let t = look.t;
    let first = row.slot == Slot::First;
    let parcel = row.slot == Slot::Parcel;

    let (rect, row_resp) = ui.allocate_exact_size(vec2(width, ROW_H), Sense::hover());
    let hot = ui.rect_contains_pointer(rect);
    if first {
        row_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &row_resp, parts::ROW);
    }
    let fill = if row.active {
        Some(t.selection_1)
    } else if hot {
        Some(t.wash_1)
    } else {
        None
    };
    if let Some(c) = fill {
        ui.painter().rect_filled(rect, CornerRadius::same(4), c);
    }

    let inner = rect.shrink2(vec2(INSET, 0.0));
    ui.scope_builder(UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)), |ui| {
        ui.spacing_mut().item_spacing = vec2(GAP, 0.0);

        // The done check: the Checkbox widget, with a press or Space toggling the task.
        let checked = row.done;
        let label = if checked { "Mark not done" } else { "Mark done" };
        let check_rect = ui
            .scope(|ui| {
                Checkbox::new(CheckboxProps { checked, ariaLabel: label.into(), ..Default::default() }).ui(ui, look);
            })
            .response
            .rect;
        let check_part = if first {
            Some(parts::ROWCHECK)
        } else if parcel {
            Some(parts::PARCELCHECK)
        } else {
            None
        };
        let check = tag(ui, egui::Id::unique(("multiinspector.check", row.id.as_str())), check_rect, Sense::click(), check_part, || WidgetInfo::selected(Role::CheckBox, true, checked, label));
        let enter = check.has_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        let space = check.has_focus() && ui.input(|i| i.key_pressed(Key::Space));
        if (check.clicked() && !enter) || space {
            acts.push(Act::Check(row.id.clone(), !row.done));
        }
        if check.has_focus() {
            ui.painter().rect_stroke(check_rect.expand(2.0), CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }

        // Open, the list badge and the title, from the right: the title takes what is left.
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Open fades in while its row is hovered (the web's reveal="row").
            let (open_rect, open) = ui.allocate_exact_size(vec2(ICON, ICON), Sense::click());
            let reveal = ui.ctx().animate_bool_with_time(egui::Id::unique(("multiinspector.reveal", row.id.as_str())), hot || open.has_focus(), 0.18);
            if open.hovered() {
                ui.painter().rect_filled(open_rect, CornerRadius::same(4), t.wash_1.gamma_multiply(reveal));
            }
            let glyph = icon("open_in_full", 13.0);
            let color = if open.hovered() { t.content_2 } else { t.content_3 };
            ui.painter().text(open_rect.center(), Align2::CENTER_CENTER, glyph.text(), FontId::new(13.0, FontFamily::Name("Icons".into())), color.gamma_multiply(reveal));
            open.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Open"));
            if first {
                part(ui, &open, parts::ROWOPEN);
            }
            if open.clicked() {
                acts.push(Act::Open(row.id.clone()));
            }

            // The list name: the Badge widget (outline, md).
            if let Some(list) = &row.list {
                let badge = BadgeProps { children: list.clone(), ..Default::default() };
                let badge_rect = ui
                    .scope(|ui| {
                        Badge::new(badge).ui(ui, look);
                    })
                    .response
                    .rect;
                if first {
                    tag(ui, egui::Id::unique("multiinspector.list"), badge_rect, Sense::hover(), Some(parts::ROWLISTBADGE), || value_info(list, list));
                }
            }

            // The title: a press peeks the task. Done titles are muted and struck through.
            let w = ui.available_width().max(0.0);
            let (title_rect, title) = ui.allocate_exact_size(vec2(w, TITLE_H), Sense::click());
            let title_part = match row.slot {
                Slot::First => Some(parts::ROWTITLE),
                Slot::Parcel => Some(parts::PARCELTITLE),
                Slot::Last => Some(parts::LASTROWTITLE),
                Slot::Other => None,
            };
            title.widget_info(|| WidgetInfo::labeled(Role::Button, true, &row.title));
            if let Some(p) = title_part {
                part(ui, &title, p);
            }
            if title.clicked() {
                acts.push(Act::Peek(row.id.clone()));
            }
            let color = if row.done { t.content_3 } else { t.content_1 };
            let galley = ui.painter().layout_no_wrap(row.title.clone(), FontId::new(rem(0.964), weight(400)), color);
            let (gw, gh) = (galley.size().x, galley.size().y);
            let painter = ui.painter().with_clip_rect(title_rect);
            let mid = title_rect.center().y;
            painter.galley(pos2(title_rect.left(), mid - gh / 2.0), galley, color);
            if row.done {
                painter.line_segment([pos2(title_rect.left(), mid), pos2(title_rect.left() + gw, mid)], Stroke::new(1.0, color));
            }
        });
    });
}

/// A line of text the contract reads: its label is the text, in Inter at `px` and `w`.
fn line(ui: &mut egui::Ui, s: &str, px: f32, w: u16, color: Color32, line_h: f32, part_id: &str) -> Response {
    let galley = ui.painter().layout_no_wrap(s.to_owned(), FontId::new(px, weight(w)), color);
    let (gw, gh) = (galley.size().x, galley.size().y);
    let (rect, response) = ui.allocate_exact_size(vec2(gw, line_h), Sense::hover());
    ui.painter().galley(pos2(rect.left(), rect.center().y - gh / 2.0), galley, color);
    response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, s));
    part(ui, &response, part_id);
    response
}

/// The "Selected" section label: the web's uppercased SectionHeader, with a rule trailing it.
fn section_label(ui: &mut egui::Ui, look: Look) {
    let t = look.t;
    let galley = ui.painter().layout_no_wrap("SELECTED".to_owned(), FontId::new(rem(0.75), weight(600)), t.content_3);
    let (gw, gh) = (galley.size().x, galley.size().y);
    let w = ui.available_width().max(0.0);
    let (rect, response) = ui.allocate_exact_size(vec2(w, rem(1.0)), Sense::hover());
    let mid = rect.center().y;
    let rule_from = rect.left() + gw + GAP;
    ui.painter().galley(pos2(rect.left(), mid - gh / 2.0), galley, t.content_3);
    ui.painter().line_segment([pos2(rule_from, mid), pos2(rect.right(), mid)], Stroke::new(1.0, t.edge_1));
    response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Selected"));
    part(ui, &response, parts::SECTIONLABEL);
}

/// Tags the node over `rect` with a sense, its role and label, and the contract part if it has one.
fn tag(ui: &egui::Ui, id: egui::Id, rect: Rect, sense: Sense, part_id: Option<&str>, info: impl Fn() -> WidgetInfo) -> Response {
    let response = ui.interact(rect, id, sense);
    response.widget_info(info);
    if let Some(p) = part_id {
        part(ui, &response, p);
    }
    response
}

/// A generic node whose label is `label` and whose value (its text) is `value`.
fn value_info(label: &str, value: &str) -> WidgetInfo {
    let mut info = WidgetInfo::labeled(Role::GenericContainer, true, label);
    info.current_text_value = Some(value.to_owned());
    info
}

/// Applies what the presses asked for, on the app.
fn apply(app: &mut App, acts: Vec<Act>) {
    for act in acts {
        match act {
            Act::Check(id, done) => {
                if let Err(e) = app.data.update_task(&id, None, None, Some(done)) {
                    app.refused(e);
                }
            }
            Act::Peek(id) => app.ui.preview = Some(id),
            Act::Open(id) => {
                app.ui.selection = vec![id.clone()];
                app.ui.preview = Some(id);
            }
            Act::Clear => {
                app.ui.selection.clear();
                app.ui.preview = None;
            }
        }
    }
}

/// The test's mount: the app on the props' data set, with the Errands list open, the named tasks
/// selected in list order, and the last of them peeked (as the web harness sets it up).
struct Mount {
    app: App,
    screen: MultiInspector,
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
    let p: MultiinspectorProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    app.open_named(Some("Home"), Some(p.list.as_str()));
    let titles: &[&str] = if p.selection == "two" { &TWO[..] } else { &FOUR[..] };
    if let Some(list) = app.active_list() {
        let picked: Vec<Id> = app
            .data
            .tasks_of(&list)
            .into_iter()
            .filter(|t| t.parent_id.is_none() && titles.contains(&t.title.as_str()))
            .map(|t| t.id.clone())
            .collect();
        app.ui.preview = picked.last().cloned();
        app.ui.selection = picked;
    }
    Ok(Box::new(Mount { app, screen: MultiInspector::new() }))
}
