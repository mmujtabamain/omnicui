//! contract: omnicui/contracts/commandlist.contract.yaml (id: ui.commandlist). The palette's results list: command groups and rows in a
//! scroll region that keeps the active row in view; content "empty" shows only the empty message.
use crate::omnicui::commandlist::{parts, CommandlistEvent, CommandlistProps};
use crate::{part, rem, weight, Look};
use egui::{Align2, CornerRadius, FontId, Role, Sense, WidgetInfo};

/// A row is 2rem tall (h-[2rem]); its fill is inset 6 px from the list's edges (mx-1.5) and its text 8 px in (px-2).
const ROW_H: f32 = rem(2.0);
const ROW_INSET: f32 = 6.0;
const ROW_TEXT_X: f32 = 8.0;
/// A group heading is inset 14 px (px-3.5), 8 px above and 4 px below its text.
const HEADING_X: f32 = 14.0;
const HEADING_TOP: f32 = 8.0;
const HEADING_BOTTOM: f32 = 4.0;
/// The scroll region's padding (py-3), and the empty message's vertical padding (py-6).
const LIST_PAD: f32 = 12.0;
const EMPTY_PAD_Y: f32 = 24.0;
const RADIUS: u8 = 4;

/// A command row: the id it picks and the label it shows.
struct Row {
    id: String,
    label: String,
}

/// A group: its heading and its rows.
struct Group {
    heading: &'static str,
    rows: Vec<Row>,
}

/// The fixture groups (the web harness's buildGroups): rows cmd-01 ... cmd-NN, the first half in Tasks and the rest in Lists.
fn build_groups(row_count: i64) -> Vec<Group> {
    let total = row_count.clamp(0, 100) as usize;
    let task_count = (total + 1) / 2;
    let row = |i: usize, noun: &str| Row {
        id: format!("cmd-{i:02}"),
        label: format!("{noun} action {i:02}"),
    };
    let tasks: Vec<Row> = (1..=task_count).map(|i| row(i, "Task")).collect();
    let lists: Vec<Row> = (task_count + 1..=total).map(|i| row(i, "List")).collect();
    let mut groups = Vec::new();
    if !tasks.is_empty() {
        groups.push(Group { heading: "Tasks", rows: tasks });
    }
    if !lists.is_empty() {
        groups.push(Group { heading: "Lists", rows: lists });
    }
    groups
}

pub struct CommandList {
    p: CommandlistProps,
    /// The active row and the fixture the list last scrolled for, so that a scroll away from the active row is kept.
    scrolled_for: Option<String>,
}

impl CommandList {
    pub fn new(p: CommandlistProps) -> Self {
        Self { p, scrolled_for: None }
    }

    /// One frame: draws the list (or the empty message) and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<CommandlistEvent> {
        let mut events = vec![];
        // Rows and groups stack with no gap, as the web's flex column does. The scope keeps this off the caller's spacing.
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            if self.p.content == "empty" {
                self.empty(ui, look);
            } else {
                self.rows(ui, look, &mut events);
            }
        });
        events
    }

    /// The empty message alone: a polite status node carrying emptyMessage, centred in the palette.
    fn empty(&self, ui: &mut egui::Ui, look: Look) {
        let width = ui.available_width();
        let font = FontId::new(rem(0.929), weight(400));
        let galley = ui.painter().layout_no_wrap(self.p.emptyMessage.clone(), font, look.t.content_2);
        let height = 2.0 * EMPTY_PAD_Y + galley.size().y;
        let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
        let message = self.p.emptyMessage.clone();
        response.widget_info(|| WidgetInfo::labeled(Role::Status, true, &message));
        part(ui, &response, parts::EMPTY);
        let pos = rect.center() - galley.size() * 0.5;
        ui.painter().galley(pos, galley, look.t.content_2);
    }

    /// The populated list: groups with headings, and their rows, in a region at most 60vh tall.
    fn rows(&mut self, ui: &mut egui::Ui, look: Look, events: &mut Vec<CommandlistEvent>) {
        let t = look.t;
        let groups = build_groups(self.p.rowCount);
        let first_id = groups.first().and_then(|g| g.rows.first()).map(|r| r.id.clone());
        let last_id = groups.last().and_then(|g| g.rows.last()).map(|r| r.id.clone());
        let active = self.p.activeId.clone();

        // Scroll the active row into view (the nearest edge) once for each change of it or of the fixture.
        let key = format!("{}|{}", self.p.activeId, self.p.rowCount);
        let scroll_to = if !active.is_empty() && self.scrolled_for.as_deref() != Some(key.as_str()) {
            Some(active.clone())
        } else {
            None
        };
        self.scrolled_for = Some(key);

        // max-h-[60vh] of the window.
        let max_h = ui.ctx().content_rect().height() * 0.6;


        let out = egui::ScrollArea::vertical()
            .id_salt("list")
            .max_height(max_h)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                let width = ui.available_width();
                let font_row = FontId::new(rem(0.929), weight(400));
                let head_h = HEADING_TOP + rem(0.786) * 1.1 + HEADING_BOTTOM;
                ui.add_space(LIST_PAD);
                for (gi, group) in groups.iter().enumerate() {
                    let font_head = FontId::new(rem(0.786), weight(500));
                    let (hrect, hresp) = ui.allocate_exact_size(egui::vec2(width, head_h), Sense::hover());
                    ui.painter().text(
                        hrect.left_top() + egui::vec2(HEADING_X, HEADING_TOP),
                        Align2::LEFT_TOP,
                        group.heading,
                        font_head,
                        t.content_2,
                    );
                    hresp.widget_info(|| WidgetInfo::labeled(Role::Heading, true, group.heading));
                    if gi == 0 {
                        part(ui, &hresp, parts::FIRSTGROUPHEADING);
                    }

                    let mut bounds = hrect;
                    for row in &group.rows {
                        // The row's hit area is its full line; its fill and text sit inside the 6 px margins.
                        let (outer, response) = ui.allocate_exact_size(egui::vec2(width, ROW_H), Sense::click());
                        let rect = outer.shrink2(egui::vec2(ROW_INSET, 0.0));
                        let is_active = row.id == active;
                        let fill = if is_active {
                            Some(t.selection_1)
                        } else if response.hovered() {
                            Some(t.wash_1)
                        } else {
                            None
                        };
                        let painter = ui.painter();
                        if let Some(fill) = fill {
                            painter.rect_filled(rect, CornerRadius::same(RADIUS), fill);
                        }
                        painter.text(
                            rect.left_center() + egui::vec2(ROW_TEXT_X, 0.0),
                            Align2::LEFT_CENTER,
                            &row.label,
                            font_row.clone(),
                            t.content_1,
                        );

                        response.widget_info(|| WidgetInfo::selected(Role::Button, true, is_active, &row.label));
                        // The active row is the option the palette points at: its node carries the selected state.
                        ui.ctx().accesskit_node_builder(response.id, |node| node.set_selected(is_active));
                        let slot = if first_id.as_deref() == Some(row.id.as_str()) {
                            Some(parts::FIRSTROW)
                        } else if last_id.as_deref() == Some(row.id.as_str()) {
                            Some(parts::LASTROW)
                        } else {
                            None
                        };
                        if let Some(slot) = slot {
                            part(ui, &response, slot);
                        }
                        if scroll_to.as_deref() == Some(row.id.as_str()) {
                            ui.scroll_to_rect(rect, None);
                        }
                        if response.clicked() {
                            events.push(CommandlistEvent::Picked { id: row.id.clone() });
                        }
                        // Rows are options, not tab stops: focus stays in the palette input.
                        if response.has_focus() {
                            ui.memory_mut(|m| m.surrender_focus(response.id));
                        }
                        bounds = bounds.union(outer);
                    }

                    // The group: labelled by its heading, spanning the heading and its rows.
                    let group_resp = ui.interact(bounds, ui.scope_id().with(("group", gi)), Sense::hover());
                    group_resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, group.heading));
                    if gi == 0 {
                        part(ui, &group_resp, parts::FIRSTGROUP);
                    }
                }
                ui.add_space(LIST_PAD);
            });

        // The scrolling region itself, over the viewport the rows are seen through.
        let list = ui.interact(out.inner_rect, ui.scope_id().with("list-node"), Sense::hover());
        list.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &list, parts::LIST);
    }
}

crate::mounted!(CommandList, CommandlistProps);
