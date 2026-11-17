//! contract: omnicui/contracts/tasktreerow.contract.yaml (id: ui.tasktreerow). One row of the task tree: a round check (select, or done while the parent holds the done action), the title, indent by depth, selection and drop feedback, and the tree's keys.
use crate::omnicui::checkbox::{parts as checkbox_parts, CheckboxEvent, CheckboxProps};
use crate::omnicui::tasktreerow::{parts, TasktreerowEvent, TasktreerowProps};
use crate::widgets::checkbox::Checkbox;
use crate::{as_part, part, rem, weight, Look};
use egui::text::{LayoutJob, TextFormat};
use egui::{pos2, vec2, CornerRadius, EventFilter, FontId, Key, Rect, Role, Sense, Stroke, StrokeKind, UiBuilder, WidgetInfo};

/// The row is 30px high (the web's min height), its round check 18px, 9px from the title; the row keeps 10px on the right.
const H: f32 = rem(2.143);
const CHECK: f32 = 18.0;
const GAP: f32 = 9.0;
const PAD_RIGHT: f32 = 10.0;
/// The web's padding-left: 20px, plus 24px for each depth.
const INDENT: f32 = 20.0;
const INDENT_STEP: f32 = 24.0;
/// The focus ring and the drop line are 2px; the row's corners are 5px.
const LINE: f32 = 2.0;
const RADIUS: u8 = 5;

/// One row of the task tree. Its done and selected state follow the presses, as the web's harness keeps them.
pub struct TaskTreeRow {
    p: TasktreerowProps,
    done: bool,
    selected: bool,
    expanded: bool,
}

impl TaskTreeRow {
    pub fn new(p: TasktreerowProps) -> Self {
        Self { done: p.done, selected: p.selected, expanded: p.expanded, p }
    }

    /// One frame: draws the row and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<TasktreerowEvent> {
        let mut events = vec![];
        let t = look.t;
        let title = self.p.title.clone();
        let drop = self.p.dropIndicator.clone();
        let done_action = self.p.checkAction == "done";
        let has_children = self.p.hasChildren;
        let alpha = if self.p.dragging { 0.4 } else { 1.0 };
        let highlighted = self.selected || self.p.picked;
        let checked = self.done || (!done_action && highlighted);
        let intent = if self.done || done_action { "done" } else { "select" };
        let check_label = if done_action { format!("Mark {title} done") } else { format!("Select {title}") };
        let indent = INDENT + self.p.depth.max(0) as f32 * INDENT_STEP;
        let radius = CornerRadius::same(RADIUS);
        let activate = |platform_modifier: bool, shift: bool| TasktreerowEvent::Activate { platformModifier: platform_modifier, shift };
        // The platform modifier: Command on macOS, Control elsewhere (egui's `command` is Control on other platforms too).
        let platform = |m: egui::Modifiers| if cfg!(target_os = "macos") { m.command } else { m.command || m.ctrl };

        // The row's node is registered before the check, so a press on the check is the check's, not the row's.
        let width = if ui.available_width().is_finite() { ui.available_width() } else { 240.0 };
        let (rect, _) = ui.allocate_exact_size(vec2(width, H), Sense::hover());
        let row = ui.interact(rect, ui.scope_id().with("ui.tasktreerow.row"), Sense::click());
        let hovered = ui.rect_contains_pointer(rect);
        // A press on the row takes focus, as the web's tabbable row does.
        if row.clicked() {
            row.request_focus();
        }
        let focused = row.has_focus();
        if focused {
            // The arrows and Escape belong to the tree, not to egui's spatial focus movement.
            let filter = EventFilter { tab: false, horizontal_arrows: true, vertical_arrows: true, escape: false };
            ui.memory_mut(|m| m.set_focus_lock_filter(row.id, filter));
        }

        // Keys on the focused row. egui also turns Enter and Space into a click, so pointer clicks are told apart by the release.
        let (m, enter, space, down, up, right, left, f10) = ui.input(|i| {
            (
                i.modifiers,
                focused && i.key_pressed(Key::Enter),
                focused && i.key_pressed(Key::Space),
                focused && i.key_pressed(Key::ArrowDown),
                focused && i.key_pressed(Key::ArrowUp),
                focused && i.key_pressed(Key::ArrowRight),
                focused && i.key_pressed(Key::ArrowLeft),
                focused && i.key_pressed(Key::F10),
            )
        });
        let pointer_click = row.clicked() && ui.input(|i| i.pointer.primary_released());

        if pointer_click {
            events.push(activate(platform(m), m.shift));
        }
        if row.double_clicked() || (enter && platform(m)) {
            self.done = !self.done;
            events.push(TasktreerowEvent::ToggleDone {});
        }
        if enter && !platform(m) {
            events.push(activate(false, false));
        }
        if space {
            events.push(activate(platform(m), false));
        }
        if (down || up) && m.shift {
            events.push(activate(false, true));
        }
        if right && has_children && !self.expanded {
            self.expanded = true;
            events.push(TasktreerowEvent::ToggleExpanded { expanded: true });
        }
        if left && has_children && self.expanded {
            self.expanded = false;
            events.push(TasktreerowEvent::ToggleExpanded { expanded: false });
        }
        if row.secondary_clicked() || (f10 && m.shift) {
            events.push(TasktreerowEvent::ContextMenu {});
        }

        // Row fill: the selection while picked or previewed, or the wash under the pointer when nothing else is shown.
        let painter = ui.painter().clone();
        let fill = if highlighted || drop == "nest" {
            Some(t.selection_1)
        } else if hovered && drop == "none" {
            Some(t.wash_1)
        } else {
            None
        };
        if let Some(c) = fill {
            painter.rect_filled(rect, radius, c.gamma_multiply(alpha));
        }
        if drop == "nest" {
            painter.rect_stroke(rect, radius, Stroke::new(LINE, t.accent_500.gamma_multiply(alpha)), StrokeKind::Inside);
        }

        // The round check, drawn by the shared checkbox at its web position, and reported as the check part.
        let check_rect = Rect::from_min_size(pos2(rect.left() + indent, rect.center().y - CHECK / 2.0), vec2(CHECK, CHECK));
        let props: CheckboxProps = serde_json::from_value(serde_json::json!({
            "size": "md",
            "shape": "round",
            "intent": intent,
            "checked": checked,
            "ariaLabel": check_label,
        }))
        .expect("checkbox props");
        let mut check = Checkbox::new(props);
        let mut cu = ui.new_child(UiBuilder::new().id_salt(row.id.with("check")).max_rect(check_rect));
        cu.multiply_opacity(alpha);
        let check_events = as_part(&mut cu, &[(checkbox_parts::CONTROL, parts::CHECK)], |cu| check.ui(cu, look));
        if check_events.iter().any(|e| matches!(e, CheckboxEvent::CheckedChange { .. })) {
            if done_action {
                self.done = !self.done;
                events.push(TasktreerowEvent::Check { action: "done".to_owned() });
            } else {
                self.selected = !self.selected;
                events.push(TasktreerowEvent::Check { action: "select".to_owned() });
            }
        }

        // The title: 14px Inter Medium, muted and struck through when done, cut with an ellipsis at the row's edge.
        let title_x = check_rect.right() + GAP;
        let title_w = (rect.right() - PAD_RIGHT - title_x).max(0.0);
        let ink = (if self.done { t.content_3 } else { t.content_1 }).gamma_multiply(alpha);
        let mut job = LayoutJob::single_section(
            title.clone(),
            TextFormat {
                font_id: FontId::new(rem(1.0), weight(500)),
                color: ink,
                strikethrough: if self.done { Stroke::new(1.0, ink) } else { Stroke::NONE },
                ..Default::default()
            },
        );
        job.wrap.max_width = title_w;
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        job.wrap.overflow_character = Some('…');
        let galley = ui.painter().layout_job(job);
        let title_pos = pos2(title_x, rect.center().y - galley.size().y / 2.0);
        let title_node = ui.interact(Rect::from_min_size(title_pos, galley.size()), row.id.with("title"), Sense::hover());
        title_node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &title));
        part(ui, &title_node, parts::TITLE);
        painter.galley(title_pos, galley, ink);

        // The focus ring (inset, as the web's ring-inset) and the drop line on the top or bottom edge.
        if focused {
            painter.rect_stroke(rect, radius, Stroke::new(LINE, t.focus_1.gamma_multiply(alpha)), StrokeKind::Inside);
        }
        let accent = t.accent_500.gamma_multiply(alpha);
        if drop == "before" {
            painter.rect_filled(Rect::from_min_size(rect.left_top(), vec2(width, LINE)), CornerRadius::same(0), accent);
        }
        if drop == "after" {
            painter.rect_filled(Rect::from_min_size(pos2(rect.left(), rect.bottom() - LINE), vec2(width, LINE)), CornerRadius::same(0), accent);
        }

        // The row is a tree item: a button named by its title, selected while highlighted, and expanded when it has children.
        row.widget_info(|| WidgetInfo::labeled(Role::Button, true, &title));
        let expanded = self.expanded;
        ui.ctx().accesskit_node_builder(row.id, |node| {
            node.set_selected(highlighted);
            if has_children {
                node.set_expanded(expanded);
            }
        });
        part(ui, &row, parts::ROW);

        events
    }
}

crate::mounted!(TaskTreeRow, TasktreerowProps);
