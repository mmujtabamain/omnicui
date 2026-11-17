//! contract: omnicui/contracts/toggle.contract.yaml (id: ui.toggle). A two-state button with aria-pressed for
//! panel, mode and format switches: a square icon, or a label with an icon, in three sizes.
use crate::omnicui::toggle::{parts, ToggleEvent, ToggleProps};
use crate::{icon, part, rem, weight, Look};
use egui::{pos2, Align2, CornerRadius, FontFamily, FontId, Role, Sense, Stroke, StrokeKind, WidgetInfo};

pub struct Toggle {
    p: ToggleProps,
    /// `pressed` was in the props: the app owns the state, and a press only asks for the next one.
    controlled: bool,
    /// The uncontrolled state, started from `defaultPressed`.
    on: bool,
}

impl Toggle {
    pub fn new(p: ToggleProps, controlled: bool) -> Self {
        let on = p.defaultPressed;
        Self { p, controlled, on }
    }

    /// The pressed state as the toggle shows it.
    pub fn pressed(&self) -> bool {
        if self.controlled {
            self.p.pressed
        } else {
            self.on
        }
    }

    /// One frame: draws the toggle and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<ToggleEvent> {
        let mut events = vec![];
        let t = look.t;
        let disabled = self.p.disabled;
        let is_text = self.p.shape == "text";
        // Web: sm h-6 (21px) with a 15px glyph, md h-7 (24.5px) with 14px, lg h-8 (28px) with 19px.
        let (height, radius, glyph_px) = match self.p.size.as_str() {
            "md" => (rem(1.75), 5_u8, 14.0),
            "lg" => (rem(2.0), 7_u8, 19.0),
            _ => (rem(1.5), 5_u8, 15.0),
        };
        // px-1.5 is 5.25px of padding; gap-1 is 3.5px between the glyph and the label.
        let pad = rem(0.375);
        let gap = rem(0.25);
        let has_glyph = !self.p.icon.is_empty() || !self.p.pressedIcon.is_empty();
        let label_w = if is_text && !self.p.label.is_empty() {
            let label = self.p.label.clone();
            ui.painter().layout_no_wrap(label, FontId::new(12.0, weight(600)), t.content_2).size().x
        } else {
            0.0
        };
        let width = if is_text {
            let mut w = 2.0 * pad;
            if has_glyph {
                w += glyph_px;
            }
            if has_glyph && label_w > 0.0 {
                w += gap;
            }
            w + label_w
        } else {
            height
        };

        let sense = if disabled { Sense::hover() } else { Sense::click() };
        // The focus the toggle had before this frame, so a press can give it back (retainFocus).
        let prev_focus = ui.memory(|m| m.focused());
        let (rect, mut response) = ui.allocate_exact_size(egui::vec2(width, height), sense);
        if self.p.retainFocus && response.is_pointer_button_down_on() {
            let id = response.id;
            ui.memory_mut(|m| match prev_focus {
                Some(prev) => m.request_focus(prev),
                None => m.surrender_focus(id),
            });
        }
        // Enter and Space on a focused widget arrive as a click, so one check covers all three.
        if !disabled && response.clicked() {
            let next = !self.pressed();
            if !self.controlled {
                self.on = next;
            }
            events.push(ToggleEvent::PressedChange { pressed: next });
            response.mark_changed();
        }
        let pressed = self.pressed();
        let name = if !self.p.ariaLabel.is_empty() {
            self.p.ariaLabel.clone()
        } else if is_text {
            self.p.label.clone()
        } else {
            String::new()
        };
        response.widget_info(|| WidgetInfo::selected(Role::Button, !disabled, pressed, &name));
        part(ui, &response, parts::TOGGLE);
        // aria-pressed on a button reads as the node's selected state; WidgetInfo alone does not set it for a button.
        ui.ctx().accesskit_node_builder(response.id, |node| node.set_selected(pressed));

        let hovered = response.hovered() && !disabled;
        let alpha = if disabled { 0.45 } else { 1.0 };
        // Web: 0.18 s colour transition; the wash is wash-2 when on and wash-1 on hover.
        let fade = ui.ctx().animate_bool_with_time(response.id.with("fill"), pressed || hovered, 0.18);
        let wash = if pressed { t.wash_2 } else { t.wash_1 };
        let fill = wash.gamma_multiply(fade * alpha);
        let base_ink = if pressed || hovered { t.content_1 } else { t.content_2 };
        let ink = base_ink.gamma_multiply(alpha);

        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(radius), fill);
        // The focus ring is 2px of focus_1 just outside the toggle (ring-2, no offset).
        if response.has_focus() {
            painter.rect_stroke(rect.expand(1.0), CornerRadius::same(radius + 1), Stroke::new(2.0, t.focus_1), StrokeKind::Middle);
        }

        let cy = rect.center().y;
        let glyph_name = if pressed && !self.p.pressedIcon.is_empty() {
            self.p.pressedIcon.clone()
        } else {
            self.p.icon.clone()
        };
        let glyph_x = if is_text { rect.left() + pad + glyph_px / 2.0 } else { rect.center().x };
        if !glyph_name.is_empty() {
            let glyph = icon(&glyph_name, glyph_px);
            painter.text(
                pos2(glyph_x, cy),
                Align2::CENTER_CENTER,
                glyph.text(),
                FontId::new(glyph_px, FontFamily::Name("Icons".into())),
                ink,
            );
        }
        if is_text && !self.p.label.is_empty() {
            let x = rect.left() + pad + if has_glyph { glyph_px + gap } else { 0.0 };
            painter.text(pos2(x, cy), Align2::LEFT_CENTER, self.p.label.as_str(), FontId::new(12.0, weight(600)), ink);
        }

        // The shortcut tooltip, shown on hover. Absent when there is no shortcut.
        if !self.p.shortcut.is_empty() {
            let tip = tooltip_text(&self.p.shortcut);
            response.on_hover_ui(|ui| {
                let r = ui.label(crate::text(tip.clone(), 12.0, 500));
                r.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, tip.clone()));
                part(ui, &r, parts::TOOLTIP);
            });
        }
        events
    }
}

impl crate::Mounted for Toggle {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.ui(ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the toggle from a test's props. `pressed` is read from the JSON, because the generated
/// struct cannot tell an absent `pressed` (uncontrolled) from `false` (controlled).
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let controlled = props.get("pressed").is_some_and(|v| !v.is_null());
    let p: ToggleProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(Toggle::new(p, controlled)))
}

/// The shortcut as the tooltip shows it: Meta is Cmd on macOS and Ctrl elsewhere.
fn tooltip_text(shortcut: &str) -> String {
    let mac = cfg!(target_os = "macos");
    let keys: Vec<&str> = shortcut
        .split('+')
        .map(|k| match k.trim() {
            "Meta" => {
                if mac {
                    "⌘"
                } else {
                    "Ctrl"
                }
            }
            "Shift" => {
                if mac {
                    "⇧"
                } else {
                    "Shift"
                }
            }
            "Alt" => {
                if mac {
                    "⌥"
                } else {
                    "Alt"
                }
            }
            "Backslash" => "\\",
            other => other,
        })
        .collect();
    keys.join(if mac { "" } else { "+" })
}
