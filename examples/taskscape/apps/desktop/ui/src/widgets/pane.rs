//! contract: omnicui/contracts/pane.contract.yaml (id: ui.pane). One region of a pane group: a task pane, the aside host, or a fixed side column.
use crate::omnicui::pane::{parts, PaneEvent, PaneProps};
use crate::{part, Look};
use egui::{Align2, CornerRadius, FontId, Pos2, Rect, Role, Sense, UiBuilder, WidgetInfo};

/// Layout only: a region of a pane group. Task panes take a share of the area, the aside host and
/// fixed columns take a pixel width. The pane emits nothing and has no keys of its own.
pub struct Pane {
    p: PaneProps,
}

fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    v.max(lo).min(hi)
}

impl Pane {
    pub fn new(p: PaneProps) -> Self {
        Self { p }
    }

    /// The pane's width in px inside an area `area` px wide, from the web's slot rules.
    fn width(&self, area: f32) -> f32 {
        let p = &self.p;
        if p.slot == "aside" {
            let lo = if p.minSize != 0.0 { p.minSize } else { 280.0 };
            let hi = if p.maxSize != 0.0 { p.maxSize } else { 420.0 };
            return clamp(p.previewWidth, lo, hi) as f32;
        }
        if p.width > 0.0 {
            return p.width as f32;
        }
        // An unsplit primary sits at 100 and is not clamped; a split pane is clamped to its slot bounds.
        let split = p.slot == "split" || p.defaultSize < 100.0;
        let share = if p.defaultSize < 100.0 { p.defaultSize } else { clamp(p.ratio, 0.25, 0.75) * 100.0 };
        let percent = if split {
            let lo = if p.minSize != 0.0 { p.minSize } else { 25.0 };
            let hi = if p.maxSize != 0.0 { p.maxSize } else { 75.0 };
            clamp(share, lo, hi)
        } else {
            100.0
        };
        area * (percent / 100.0) as f32
    }

    /// One frame: draws the pane with nothing inside it.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<PaneEvent> {
        self.ui_with(ui, look, |_| {})
    }

    /// One frame: draws the pane and `content` inside it. A closed pane (`open: false`) is not drawn:
    /// its part stays as a 0 × 0 node, which the target reads as not visible.
    pub fn ui_with(&mut self, ui: &mut egui::Ui, look: Look, content: impl FnOnce(&mut egui::Ui)) -> Vec<PaneEvent> {
        if !self.p.open {
            let (_, response) = ui.allocate_exact_size(egui::vec2(0.0, 0.0), Sense::hover());
            response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &response, parts::PANE);
            return vec![];
        }
        let avail = ui.available_size();
        let area_w = if avail.x.is_finite() { avail.x.max(0.0) } else { 860.0 };
        let height = if avail.y.is_finite() { avail.y.max(0.0) } else { 715.0 };

        // Without shrink the pane gives way to a narrower window; with shrink (or a fixed width) it keeps its width.
        let mut width = self.width(area_w);
        if !(self.p.shrink || self.p.width > 0.0) {
            width = width.min(area_w);
        }

        let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
        response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &response, parts::PANE);

        let t = look.t;
        let mut body = rect;
        {
            let painter = ui.painter();
            if self.p.fill == "surface" {
                painter.rect_filled(rect, CornerRadius::same(0), t.surface_1);
            }
            // The hairline is part of the pane's own box (border-box), so the content is inset by it.
            if self.p.edge == "leading" {
                let line = Rect::from_min_size(rect.left_top(), egui::vec2(1.0, rect.height()));
                painter.rect_filled(line, CornerRadius::same(0), t.edge_2);
                body.min.x += 1.0;
            } else if self.p.edge == "trailing" {
                let line = Rect::from_min_size(Pos2::new(rect.right() - 1.0, rect.top()), egui::vec2(1.0, rect.height()));
                painter.rect_filled(line, CornerRadius::same(0), t.edge_2);
                body.max.x -= 1.0;
            }
        }

        ui.scope_builder(UiBuilder::new().max_rect(body), |child| {
            // The pane clips its content to its own box (overflow hidden on a surface, and always on the box edge).
            child.set_clip_rect(rect);
            content(child);
        });

        // The pane raises no events of its own.
        vec![]
    }
}

/// The harness's rows, as the web fixture renders them beside the pane.
static TASKS: [&str; 3] = ["Draft the release notes", "Review the export flow", "Plan the next sprint"];
static PREVIEW: [&str; 3] = ["Preview", "Due Friday", "Two subtasks open"];
static SECTIONS: [&str; 3] = ["General", "Shortcuts", "Appearance"];

/// Draws the harness's text rows down the pane, as the web fixture lays them out: the column is p-2
/// (8 px), each row is px-2 (8 px) by py-1.5 (30 px tall with its line), rows are gap-1 (3.5 px) apart.
/// The text is 13 px Inter in content-1.
fn rows_ui(ui: &mut egui::Ui, look: Look, rows: &[&str]) {
    let pad = 8.0;
    ui.spacing_mut().item_spacing.y = 3.5;
    ui.add_space(pad);
    for row in rows {
        let width = (ui.available_width()).max(0.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 30.0), Sense::hover());
        ui.painter().text(
            Pos2::new(rect.left() + 2.0 * pad, rect.center().y),
            Align2::LEFT_CENTER,
            *row,
            FontId::new(13.0, crate::weight(400)),
            look.t.content_1,
        );
    }
}

/// The desktop mount: the pane with the harness's rows inside it, as the web harness renders them.
pub struct PaneMount {
    pane: Pane,
    rows: &'static [&'static str],
}

impl crate::Mounted for PaneMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        let rows = self.rows;
        for e in self.pane.ui_with(ui, look, |ui| rows_ui(ui, look, rows)) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the pane from a test's props. The rows follow the harness: nav shows the sections, aside the preview.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: PaneProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    let rows: &'static [&'static str] = if p.r#as == "nav" {
        &SECTIONS
    } else if p.slot == "aside" {
        &PREVIEW
    } else {
        &TASKS
    };
    Ok(Box::new(PaneMount { pane: Pane::new(p), rows }))
}
