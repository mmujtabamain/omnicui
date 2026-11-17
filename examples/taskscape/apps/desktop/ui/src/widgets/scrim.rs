//! contract: omnicui/contracts/scrim.contract.yaml (id: ui.scrim). The full-window layer behind a modal, a menu or the lightbox: a soft or strong dim, or an invisible hit area; a press on it dismisses the owner.
use crate::omnicui::scrim::{parts, ScrimEvent, ScrimProps};
use crate::{part, Look};
use egui::{Color32, CornerRadius, Role, Sense, WidgetInfo};

pub struct Scrim {
    p: ScrimProps,
}

impl Scrim {
    pub fn new(p: ScrimProps) -> Self {
        Self { p }
    }

    /// One frame: draws the layer over the window and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, _look: Look) -> Vec<ScrimEvent> {
        let mut events = vec![];
        // false: not rendered at all, so the part has no node and the target reports it as not visible.
        if !self.p.open {
            return events;
        }
        // `fixed inset-0`: the layer covers the whole window. It takes pointer presses so that nothing
        // beneath it is clicked, but it takes no keys (Escape belongs to the owner).
        let rect = ui.ctx().content_rect();
        let response = ui.interact(rect, ui.scope_id().with("scrim"), Sense::click());
        // Hidden from assistive technology: a generic node with no label or value.
        response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &response, parts::SCRIM);

        // A press with any button on the scrim dismisses, as the web's mousedown does (no button filter).
        let pressed = ui.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::PointerButton { pressed: true, .. }))
        });
        if pressed && response.contains_pointer() {
            events.push(ScrimEvent::Dismissed {});
        }

        // The web's literal dims (bg-black/30, bg-black/50), not tokens. `none` paints nothing.
        // egui has no backdrop blur, so strong-blur is the strong dim alone.
        let fill = match self.p.dim.as_str() {
            "strong-blur" => Some(Color32::from_rgba_unmultiplied(0, 0, 0, 128)),
            "none" => None,
            _ => Some(Color32::from_rgba_unmultiplied(0, 0, 0, 77)),
        };
        if let Some(fill) = fill {
            ui.ctx()
                .layer_painter(ui.layer_id())
                .rect_filled(rect, CornerRadius::ZERO, fill);
        }
        events
    }
}

crate::mounted!(Scrim, ScrimProps);
