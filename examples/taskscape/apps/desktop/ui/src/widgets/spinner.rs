//! contract: omnicui/contracts/spinner.contract.yaml (id: ui.spinner). A small inline busy indicator, 12px square, drawn in the text colour and turning while it is mounted.
use crate::omnicui::spinner::{parts, SpinnerEvent, SpinnerProps};
use crate::{part, rem, Look};
use egui::{Pos2, Role, Sense, Shape, Stroke, WidgetInfo};
use std::f32::consts::TAU;
use std::time::Duration;

/// The web's `h-[0.857rem] w-[0.857rem]`: one 12px square on the 14px root.
const SIDE: f32 = rem(0.857);
/// The gap until the next frame of the spin.
const FRAME: Duration = Duration::from_millis(16);
/// Stroke of the turning arc, in px.
const STROKE: f32 = 1.6;
/// The arc's length as a share of a full turn (the glyph is a ring with a gap).
const SWEEP: f32 = 0.75;
/// One turn per second, as the web's `animate-spin`.
const TURN_SECS: f64 = 1.0;

pub struct Spinner {
    p: SpinnerProps,
}

impl Spinner {
    pub fn new(p: SpinnerProps) -> Self {
        Self { p }
    }

    /// One frame: draws the indicator. It reports no events; the consumer unmounts it when loading ends.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<SpinnerEvent> {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(SIDE, SIDE), Sense::hover());
        let label = self.p.label.clone();
        response.widget_info(|| WidgetInfo::labeled(Role::Status, true, &label));
        part(ui, &response, parts::INDICATOR);

        // The ring's arc, turning from the current time. Kept inside the 12px square with a half-pixel margin.
        let turn = (ui.input(|i| i.time) / TURN_SECS).rem_euclid(1.0) as f32;
        let start = turn * TAU;
        let radius = SIDE / 2.0 - STROKE / 2.0 - 0.5;
        let centre = rect.center();
        let color = look.t.content_1;
        let segments = 24;
        let points: Vec<Pos2> = (0..=segments)
            .map(|k| {
                let a = start + SWEEP * TAU * k as f32 / segments as f32;
                centre + egui::vec2(a.cos(), a.sin()) * radius
            })
            .collect();
        let painter = ui.painter();
        painter.add(Shape::line(points.clone(), Stroke::new(STROKE, color)));
        // Round caps at both ends of the arc.
        painter.circle_filled(points[0], STROKE / 2.0, color);
        painter.circle_filled(points[segments], STROKE / 2.0, color);

        // The spin never stops while mounted. A timed repaint keeps it turning about 60 times a second
        // without asking for an immediate repaint every frame, so the UI can still settle in tests.
        ui.ctx().request_repaint_after(FRAME);
        vec![]
    }
}

crate::mounted!(Spinner, SpinnerProps);
