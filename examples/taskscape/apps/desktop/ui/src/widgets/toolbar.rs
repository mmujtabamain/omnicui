//! contract: omnicui/contracts/toolbar.contract.yaml (id: ui.toolbar). A horizontal group of ghost icon buttons with roving focus: the title bar tools, the Trash action strip and the bulk action bar.
use crate::omnicui::toolbar::{parts, ToolbarEvent, ToolbarProps};
use crate::{icon, part, rem, Look};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Key, Modifiers, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// A button's square side: the web's md square ghost button, 2rem of the 14px rem.
const BUTTON: f32 = rem(2.0);
/// The icon's size inside the button.
const ICON: f32 = rem(1.143);
/// The corner radius of a control, in px.
const RADIUS: u8 = 7;

/// The demo actions per placement, as the web harness renders them: three buttons, each an icon and a label.
fn actions(placement: &str) -> [(&'static str, &'static str); 3] {
    match placement {
        "strip" => [("restore_from_trash", "Restore"), ("drive_file_move", "Move"), ("delete_forever", "Delete forever")],
        "footer" => [("drive_file_move", "Move"), ("label", "Tag"), ("delete", "Delete")],
        _ => [("search", "Search"), ("filter_list", "Filter"), ("sort", "Sort")],
    }
}

pub struct Toolbar {
    p: ToolbarProps,
    /// The button that had focus at the end of the last frame: the base the arrow keys move from.
    last: Option<usize>,
}

impl Toolbar {
    pub fn new(p: ToolbarProps) -> Self {
        Self { p, last: None }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<ToolbarEvent> {
        let placement = self.p.placement.as_str();
        let strip = placement == "strip";
        let footer = placement == "footer";
        // The strip and the footer are not rendered while there is nothing to act on.
        if (strip || footer) && self.p.count < 1 {
            self.last = None;
            return vec![];
        }
        let t = look.t;
        // Web spacing: inline gap-4 px-6; strip gap-3 px-6 py-5 with an edge-2 bottom rule; footer h-9 px-6 with an edge-2 top rule, no gap.
        let pad_x = rem(1.5);
        let gap = if strip {
            rem(0.75)
        } else if footer {
            0.0
        } else {
            rem(1.0)
        };
        let height = if footer {
            rem(2.25)
        } else if strip {
            BUTTON + 2.0 * rem(1.25) + 1.0
        } else {
            BUTTON
        };
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

        let root = ui.interact(rect, ui.scope_id().with("toolbar"), Sense::hover());
        let label = self.p.ariaLabel.clone();
        root.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &label));
        part(ui, &root, parts::TOOLBAR);
        if strip {
            ui.painter().hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, t.edge_2));
        }
        if footer {
            ui.painter().hline(rect.x_range(), rect.top() + 0.5, Stroke::new(1.0, t.edge_2));
        }

        // The buttons' centre line: the strip's content sits between its top padding and its rule.
        let cy = if strip {
            rect.top() + rem(1.25) + BUTTON / 2.0
        } else if footer {
            rect.top() + 1.0 + (rect.height() - 1.0) / 2.0
        } else {
            rect.center().y
        };
        let items = actions(placement);
        let ids = [parts::FIRST, parts::SECOND, parts::THIRD];
        let base = ui.scope_id().with("button");
        let mut resp = Vec::with_capacity(items.len());
        let mut x = rect.left() + pad_x;
        for (i, &(icon_name, label)) in items.iter().enumerate() {
            let r = Rect::from_center_size(pos2(x + BUTTON / 2.0, cy), vec2(BUTTON, BUTTON));
            let response = ui.interact(r, base.with(i), Sense::click());
            // Arrow keys are the toolbar's: egui must not move focus by them, so the buttons lock them.
            let lock = egui::EventFilter { horizontal_arrows: true, vertical_arrows: true, tab: false, escape: false };
            ui.memory_mut(|m| m.set_focus_lock_filter(response.id, lock));
            let hovered = response.hovered();
            let fill = if response.is_pointer_button_down_on() {
                t.wash_2
            } else if hovered {
                t.wash_1
            } else {
                Color32::TRANSPARENT
            };
            let ink = if hovered { t.content_1 } else { t.content_2 };
            let radius = CornerRadius::same(RADIUS);
            let painter = ui.painter();
            painter.rect_filled(r, radius, fill);
            painter.text(
                r.center(),
                Align2::CENTER_CENTER,
                icon(icon_name, ICON).text(),
                FontId::new(ICON, FontFamily::Name("Icons".into())),
                ink,
            );
            if response.has_focus() {
                painter.rect_stroke(r, radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
            }
            response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
            part(ui, &response, ids[i]);
            resp.push(response);
            x += BUTTON + gap;
        }

        // Roving focus (Radix Toolbar): Left and Right move between the buttons and wrap; Home and End
        // go to the ends; Tab leaves the toolbar.
        let now = resp.iter().position(|r| r.has_focus());
        let base_idx = resp.iter().position(|r| r.clicked()).or(self.last).or(now);
        let n = resp.len();
        let target = match (base_idx, now) {
            (Some(b), Some(_)) => ui.input_mut(|i| {
                if i.consume_key(Modifiers::NONE, Key::ArrowRight) {
                    Some((b + 1) % n)
                } else if i.consume_key(Modifiers::NONE, Key::ArrowLeft) {
                    Some((b + n - 1) % n)
                } else if i.consume_key(Modifiers::NONE, Key::Home) {
                    Some(0)
                } else if i.consume_key(Modifiers::NONE, Key::End) {
                    Some(n - 1)
                } else {
                    None
                }
            }),
            _ => None,
        };
        let tab = now.is_some() && ui.input(|i| i.key_pressed(Key::Tab));
        match target {
            Some(to) => {
                resp[to].request_focus();
                self.last = Some(to);
            }
            None if tab => {
                if let Some(c) = now {
                    ui.memory_mut(|m| m.surrender_focus(resp[c].id));
                }
                self.last = None;
            }
            None => self.last = now,
        }
        vec![]
    }
}

crate::mounted!(Toolbar, ToolbarProps);
