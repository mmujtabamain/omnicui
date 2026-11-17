//! contract: omnicui/contracts/confirmbutton.contract.yaml (id: ui.confirmbutton). A button that acts only on its second activation, inline: the first press shows the confirm label, the second confirms; Escape or leaving focus cancels.
use crate::omnicui::button::{ButtonEvent, ButtonProps};
use crate::omnicui::confirmbutton::{parts, ConfirmbuttonEvent, ConfirmbuttonProps};
use crate::widgets::button::Button;
use crate::Look;
use egui::{vec2, CornerRadius, Key, Rect, Stroke, StrokeKind};

/// The web's outline button at size lg is 28px tall (rem(2)).
const HEIGHT: f32 = 28.0;

pub struct ConfirmButton {
    p: ConfirmbuttonProps,
    /// The first activation has happened: the next one confirms.
    confirming: bool,
    /// The button had keyboard focus at the end of the last frame.
    focused: bool,
    /// Tab took focus while confirming; if it comes back to this button (its only focusable widget), it is let go.
    tabbed: bool,
}

impl ConfirmButton {
    pub fn new(p: ConfirmbuttonProps) -> Self {
        Self { p, confirming: false, focused: false, tabbed: false }
    }

    /// One frame: draws the button and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<ConfirmbuttonEvent> {
        let mut events: Vec<ConfirmbuttonEvent> = vec![];
        let danger = self.p.tone == "danger";
        let avail = ui.available_width();
        let width = if avail.is_finite() { avail } else { 240.0 };
        let origin = ui.cursor().min;
        // Button allocates its response first, so the id it draws under is the next auto id.
        let id = ui.next_auto_id();
        let mut focused = ui.memory(|m| m.has_focus(id));
        let mut changed = false;

        // Escape cancels a confirm and keeps focus: egui surrenders focus on Escape, so take it back.
        if ui.input(|i| i.key_pressed(Key::Escape)) && (focused || self.focused) {
            ui.memory_mut(|m| m.request_focus(id));
            focused = true;
            if self.confirming {
                self.confirming = false;
                changed = true;
            }
        }
        // Focus leaving a confirming button disarms it, so a stale confirm cannot fire later.
        if self.focused && !focused && self.confirming {
            self.confirming = false;
            changed = true;
        }
        // Tab took focus while armed. With no other focusable widget, egui wraps focus back onto this
        // button a frame later, so let it go until a frame starts without it.
        if self.tabbed {
            if focused {
                ui.memory_mut(|m| m.surrender_focus(id));
                focused = false;
                ui.ctx().request_repaint();
            } else {
                self.tabbed = false;
            }
        }
        if focused && self.confirming && ui.input(|i| i.key_pressed(Key::Tab)) {
            self.confirming = false;
            self.tabbed = true;
            changed = true;
            ui.memory_mut(|m| m.surrender_focus(id));
            focused = false;
        }

        // Enter is the keyboard click that the danger tone must not confirm on.
        let enter = focused && ui.input(|i| i.key_pressed(Key::Enter));

        let bp = ButtonProps {
            label: if self.confirming { self.p.confirmLabel.clone() } else { self.p.label.clone() },
            variant: "outline".to_owned(),
            size: "lg".to_owned(),
            shape: "text".to_owned(),
            tone: (if danger { "danger" } else { "strong" }).to_owned(),
            width: "full".to_owned(),
            tabbable: true,
            ..Default::default()
        };
        let button_events = Button::new(bp).ui(ui, look);
        let activated = button_events.iter().any(|e| matches!(e, ButtonEvent::Activated { .. }));

        // The Button's node is this widget's control: give it the contract's part id (what `part` does).
        ui.ctx().accesskit_node_builder(id, |node| node.set_author_id(parts::BUTTON.to_owned()));

        // The confirming cue: the web's border-edge-3, or border-danger-500 for the danger tone.
        if self.confirming {
            let edge = if danger { look.t.danger_500 } else { look.t.edge_3 };
            let rect = Rect::from_min_size(origin, vec2(width, HEIGHT));
            ui.painter().rect_stroke(rect, CornerRadius::same(4), Stroke::new(1.0, edge), StrokeKind::Inside);
        }

        // An activation (click, Enter or Space) arms, then confirms. Danger ignores Enter once armed.
        if activated {
            if !self.confirming {
                self.confirming = true;
                ui.memory_mut(|m| m.request_focus(id));
                changed = true;
            } else if !(danger && enter) {
                self.confirming = false;
                events.push(ConfirmbuttonEvent::Confirmed {});
                changed = true;
            }
        }

        self.focused = ui.memory(|m| m.has_focus(id));
        if changed {
            ui.ctx().request_repaint();
        }
        events
    }
}

crate::mounted!(ConfirmButton, ConfirmbuttonProps);
