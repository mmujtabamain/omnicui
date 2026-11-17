//! contract: omnicui/contracts/separator.contract.yaml (id: ui.separator). A 1px hairline, horizontal or vertical, with menu, row and label insets; the mounted harness draws the web's fixtures around it.
use crate::omnicui::separator::{parts, SeparatorEvent, SeparatorProps};
use crate::{part, rem, text, Look};
use egui::{pos2, vec2, Align, Color32, CornerRadius, Layout, Rect, Role, Sense, Shape, Stroke, StrokeKind, WidgetInfo};

/// The rule. `ui` draws it where the layout puts it; `place` draws it at a given rect.
pub struct Separator {
    p: SeparatorProps,
}

impl Separator {
    pub fn new(p: SeparatorProps) -> Self {
        Self { p }
    }

    /// One frame: draws the rule and returns the events it raised (none).
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<SeparatorEvent> {
        if self.p.inset == "row" {
            // Absolute in its parent: at the parent's bottom edge, starting at depth * 24 + 44 px.
            let bounds = ui.max_rect();
            let left = (bounds.left() + self.p.depth as f32 * 24.0 + 44.0).min(bounds.right());
            let rect = Rect::from_min_max(pos2(left, bounds.bottom() - 1.0), pos2(bounds.right(), bounds.bottom()));
            self.place(ui, rect, look);
            return vec![];
        }
        // The menu inset is mx-[1.429rem] my-[0.857rem].
        let (ml, mr, mt, mb) = if self.p.inset == "menu" {
            (rem(1.429), rem(1.429), rem(0.857), rem(0.857))
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };
        let vertical = self.p.orientation == "vertical";
        let length = self.p.length.as_str();
        let (outer_w, outer_h, air) = if vertical {
            // w-px; xs and sm are fixed heights, full and fill stretch to the parent.
            let w = ml + 1.0 + mr;
            let h = if length == "full" || length == "fill" {
                ui.available_height()
            } else {
                let len = if length == "xs" { rem(0.786) } else { rem(1.0) };
                len + mt + mb
            };
            // fill keeps my-[0.571rem] of air above and below the stretched rule.
            (w, h, if length == "fill" { rem(0.571) } else { 0.0 })
        } else {
            // h-px, across the width the parent gives.
            (ui.available_width(), 1.0 + mt + mb, 0.0)
        };
        let (outer, _) = ui.allocate_exact_size(vec2(outer_w, outer_h), Sense::hover());
        let rect = Rect::from_min_max(
            pos2(outer.left() + ml, outer.top() + mt + air),
            pos2(outer.right() - mr, outer.bottom() - mb - air),
        );
        self.place(ui, rect, look);
        vec![]
    }

    /// Paints the rule into `rect` and tags its node with the part id.
    fn place(&self, ui: &mut egui::Ui, rect: Rect, look: Look) {
        let id = ui.scope_id().with(("rule", rect.min.x.to_bits(), rect.min.y.to_bits()));
        let response = ui.interact(rect, id, Sense::hover());
        // The contract's tests read the rule as role generic for every decorative value, including
        // decorative false (whose web role is separator), so the node is a generic container.
        response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &response, parts::RULE);
        let colour = if self.p.emphasis == "adjacent" { look.t.edge_2 } else { look.t.edge_1 };
        ui.painter().rect_filled(rect, CornerRadius::same(0), colour);
    }
}

/// The mounted widget: the web harness's fixture for the props, with the rule drawn inside it.
pub struct SeparatorMount {
    p: SeparatorProps,
    rule: Separator,
}

impl SeparatorMount {
    pub fn new(p: SeparatorProps) -> Self {
        Self { rule: Separator::new(p.clone()), p }
    }
}

impl crate::Mounted for SeparatorMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, _emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        let p = self.p.clone();
        let rule = &mut self.rule;
        if p.orientation == "vertical" {
            vertical_fixture(ui, look, rule);
        } else if p.inset == "row" {
            row_fixture(ui, look, rule);
        } else if p.inset == "label" {
            label_fixture(ui, look, rule);
        } else if p.inset == "menu" {
            menu_fixture(ui, look, rule);
        } else {
            pane_fixture(ui, look, rule);
        }
    }
}

/// Bold | rule | Italic, in an 18rem by 3rem surface.
fn vertical_fixture(ui: &mut egui::Ui, look: Look, rule: &mut Separator) {
    let t = look.t;
    let bg = ui.painter().add(Shape::Noop);
    let resp = ui.allocate_ui_with_layout(vec2(rem(18.0), rem(3.0)), Layout::left_to_right(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = rem(0.571);
        ui.add_space(rem(0.714));
        ui.label(text("Bold", rem(0.857), 400).color(t.content_2));
        rule.ui(ui, look);
        ui.label(text("Italic", rem(0.857), 400).color(t.content_2));
    });
    ui.painter().set(bg, Shape::rect_filled(resp.response.rect, CornerRadius::same(5), t.surface_3));
}

/// A task row: the text, and the rule at its bottom edge, starting at the depth's offset.
fn row_fixture(ui: &mut egui::Ui, look: Look, rule: &mut Separator) {
    let t = look.t;
    let bg = ui.painter().add(Shape::Noop);
    let resp = ui.allocate_ui_with_layout(vec2(rem(18.0), rem(2.5)), Layout::left_to_right(Align::Center), |ui| {
        ui.add_space(rem(0.857));
        ui.label(text("Draft the brief", rem(0.929), 400).color(t.content_1));
        rule.ui(ui, look);
    });
    ui.painter().set(bg, Shape::rect_filled(resp.response.rect, CornerRadius::same(0), t.surface_2));
}

/// The section label, then the rule running from the label to the pane edge.
fn label_fixture(ui: &mut egui::Ui, look: Look, rule: &mut Separator) {
    let t = look.t;
    ui.vertical(|ui| {
        ui.set_min_width(rem(18.0));
        ui.set_max_width(rem(18.0));
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = rem(0.571);
            ui.label(text("ATTACHMENTS", rem(0.786), 500).color(t.content_3));
            rule.ui(ui, look);
        });
    });
}

/// A menu panel: Rename, the inset rule, Delete.
fn menu_fixture(ui: &mut egui::Ui, look: Look, rule: &mut Separator) {
    let t = look.t;
    let bg = ui.painter().add(Shape::Noop);
    let resp = ui.vertical(|ui| {
        ui.set_min_width(rem(18.0));
        ui.set_max_width(rem(18.0));
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.add_space(rem(0.429));
        line(ui, "Rename", t.content_1, rem(0.357));
        rule.ui(ui, look);
        line(ui, "Delete", t.content_1, rem(0.357));
        ui.add_space(rem(0.429));
    });
    ui.painter().set(bg, Shape::rect_filled(resp.response.rect, CornerRadius::same(10), t.surface_3));
}

/// A pane: a header, the rule, a body, inside a bordered panel.
fn pane_fixture(ui: &mut egui::Ui, look: Look, rule: &mut Separator) {
    let t = look.t;
    let bg = ui.painter().add(Shape::Noop);
    let resp = ui.vertical(|ui| {
        ui.set_min_width(rem(18.0));
        ui.set_max_width(rem(18.0));
        ui.spacing_mut().item_spacing.y = 0.0;
        line(ui, "Pane header", t.content_1, rem(0.571));
        rule.ui(ui, look);
        line(ui, "Pane body", t.content_2, rem(0.571));
    });
    let rect = resp.response.rect;
    ui.painter().set(bg, Shape::rect_filled(rect, CornerRadius::same(10), t.surface_2));
    ui.painter().rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0, t.edge_1), StrokeKind::Inside);
}

/// A line of 13px text indented by 0.857rem, with `pad` of space above and below.
fn line(ui: &mut egui::Ui, label: &str, colour: Color32, pad: f32) {
    ui.add_space(pad);
    ui.horizontal(|ui| {
        ui.add_space(rem(0.857));
        ui.label(text(label, rem(0.929), 400).color(colour));
    });
    ui.add_space(pad);
}

/// Builds the harness from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: SeparatorProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(SeparatorMount::new(p)))
}
