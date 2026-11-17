//! contract: omnicui/contracts/taskrow.contract.yaml (id: ui.taskrow). One task line: a round done check, the title (muted and struck through when done), and a remove button that is dimmed until it is hovered.
use crate::omnicui::row::RowProps;
use crate::omnicui::taskrow::{parts, TaskrowEvent, TaskrowProps};
use crate::widgets::row::Row;
use crate::{icon, part, rem, weight, Look};
use egui::text::{LayoutJob, TextFormat};
use egui::{vec2, Align2, CornerRadius, FontFamily, FontId, Key, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// The remove button's box: the web's sm icon button (21px high and wide).
const REMOVE_PX: f32 = 21.0;
/// The hover fade of the remove button, the web's 0.18 s fast duration.
const FAST: f32 = 0.18;

/// The task line. It holds the done state, which a press on the check changes.
pub struct TaskRow {
    p: TaskrowProps,
}

impl TaskRow {
    pub fn new(p: TaskrowProps) -> Self {
        Self { p }
    }

    /// One frame: draws the row and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<TaskrowEvent> {
        let t = look.t;
        let title = self.p.title.clone();
        let mut done = self.p.done;
        let mut removed = false;

        // The contract's `row` node sits under the row's own nodes, so it is registered before the row
        // draws; it covers the row's height (a 2.25rem line, one title).
        let rect = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), rem(2.25)));
        let row_node = ui.interact(rect, ui.scope_id().with("row"), Sense::hover());
        row_node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &row_node, parts::ROW);

        // Inset, medium density: the 2.25rem line, with the surface's own fill and the 14px side padding.
        let row = Row::new(RowProps { shape: "inset".to_owned(), density: "md".to_owned(), ..Default::default() });
        row.show(ui, look, |ui, _hovered| {
            let gap = ui.spacing().item_spacing.x;

            // The round done check, 16px (the web's sm checkbox). Space toggles it; Enter does not.
            let side = rem(1.143);
            let (check_rect, check) = ui.allocate_exact_size(vec2(side, side), Sense::click());
            let enter = check.has_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            let space = check.has_focus() && ui.input(|i| i.key_pressed(Key::Space));
            if (check.clicked() && !enter) || space {
                done = !done;
            }
            let check_label = format!("Mark {title} done");
            check.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, done, &check_label));
            part(ui, &check, parts::DONE);
            let painter = ui.painter();
            let fill = if done { t.done_lamp_1 } else { t.surface_3 };
            let edge = if done { t.done_lamp_1 } else { t.content_1 };
            painter.rect(check_rect, CornerRadius::same(8), fill, Stroke::new(1.0, edge), StrokeKind::Inside);
            if done {
                painter.text(
                    check_rect.center(),
                    Align2::CENTER_CENTER,
                    icon("check", 12.0).text(),
                    FontId::new(12.0, FontFamily::Name("Icons".into())),
                    t.on_accent,
                );
            }
            if check.has_focus() {
                painter.rect_stroke(check_rect.expand(2.0), CornerRadius::same(10), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
            }

            // The title takes the room left of the remove button (flex-1, min-w-0). One line, Inter 14px
            // at weight 500; muted with a strikethrough when done.
            let title_w = (ui.available_width() - REMOVE_PX - gap).max(0.0);
            let color = if done { t.content_3 } else { t.content_1 };
            let format = TextFormat {
                font_id: FontId::new(14.0, weight(500)),
                color,
                strikethrough: if done { Stroke::new(1.0, t.content_3) } else { Stroke::NONE },
                ..Default::default()
            };
            let mut job = LayoutJob::single_section(title.clone(), format);
            job.wrap.max_width = title_w;
            job.wrap.max_rows = 1;
            job.wrap.break_anywhere = true;
            job.wrap.overflow_character = Some('…');
            let galley = ui.painter().layout_job(job);
            let (_, heading) = ui.allocate_exact_size(vec2(title_w, galley.size().y), Sense::hover());
            heading.widget_info(|| WidgetInfo::labeled(Role::Heading, true, &title));
            part(ui, &heading, parts::TITLE);
            ui.painter().galley(heading.rect.min, galley, color);

            // The remove button: a 21px square, icon close, dimmed to 60% until it is hovered.
            let (rm_rect, rm) = ui.allocate_exact_size(vec2(REMOVE_PX, REMOVE_PX), Sense::click());
            let rm_label = format!("Remove {title}");
            rm.widget_info(|| WidgetInfo::labeled(Role::Button, true, &rm_label));
            part(ui, &rm, parts::REMOVE);
            if rm.clicked() {
                removed = true;
            }
            let reveal = ui.ctx().animate_bool_with_time(rm.id.with("reveal"), rm.hovered(), FAST);
            let alpha = 0.6 + 0.4 * reveal;
            let fg = if rm.hovered() { t.content_2 } else { t.content_3 };
            let painter = ui.painter();
            if rm.hovered() {
                painter.rect_filled(rm_rect, CornerRadius::same(4), t.wash_1.gamma_multiply(alpha));
            }
            painter.text(
                rm_rect.center(),
                Align2::CENTER_CENTER,
                icon("close", 14.0).text(),
                FontId::new(14.0, FontFamily::Name("Icons".into())),
                fg.gamma_multiply(alpha),
            );
            if rm.has_focus() {
                painter.rect_stroke(rm_rect, CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
            }

            // The status: the sr-only "Done" text, present only while done. It is a 1px node, not drawn.
            if done {
                let node = ui.interact(Rect::from_min_size(ui.max_rect().min, vec2(1.0, 1.0)), ui.scope_id().with("status"), Sense::hover());
                node.widget_info(|| WidgetInfo::labeled(Role::Status, true, "Done"));
                part(ui, &node, parts::STATUS);
            }
        });

        let mut events = vec![];
        if done != self.p.done {
            self.p.done = done;
            events.push(TaskrowEvent::Toggled { done });
        }
        if removed {
            events.push(TaskrowEvent::Removed { title });
        }
        events
    }
}

/// The harness: the task line in a list (the web test wraps it in a role=list div). The list is the
/// fixture around the widget; it draws no part of the contract.
pub struct TaskRowMount {
    row: TaskRow,
}

impl TaskRowMount {
    pub fn new(p: TaskrowProps) -> Self {
        Self { row: TaskRow::new(p) }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<TaskrowEvent> {
        // The list node is registered before the row, so the row's nodes sit above it.
        let rect = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), rem(2.25)));
        let list = ui.interact(rect, ui.scope_id().with("list"), Sense::hover());
        list.widget_info(|| WidgetInfo::new(Role::List));
        self.row.ui(ui, look)
    }
}

impl crate::Mounted for TaskRowMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.ui(ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the task line from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: TaskrowProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(TaskRowMount::new(p)))
}
