//! contract: omnicui/contracts/dropindicator.contract.yaml (id: ui.dropindicator). Insertion marker drawn while a drag is over a drop target: a line with a dot before or after a task row, a line at the list root, or a bar on a tab divider, in the neutral content ink (not accent).
use crate::omnicui::dropindicator::{parts, DropindicatorProps};
use crate::{part, rem, weight, Look};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontId, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// The line of a row or the root: h-0.5, full width of the list.
const LINE_H: f32 = rem(0.125);
/// The dot at the line's left end: size-2.
const DOT: f32 = 8.0;
/// A task row of the fixture: h-[1.75rem].
const ROW_H: f32 = rem(1.75);
/// The tab bar: h-6 w-1 (1.5rem by 0.25rem), fully rounded.
const BAR_H: f32 = rem(1.5);
const BAR_W: f32 = rem(0.25);
/// The fixtures: w-[16rem]; padding and horizontal padding px-2 (8px); gap-1 (4px).
const WIDTH: f32 = rem(16.0);
const PAD: f32 = 8.0;
const GAP: f32 = 4.0;
/// The tab strip: h-[2.5rem].
const TAB_H: f32 = rem(2.5);
/// The fixture text: text-[0.929rem].
const TEXT: f32 = 13.0;
/// Radii: a task row is a field, the tab strip is a control.
const FIELD_RADIUS: u8 = 5;
const CONTROL_RADIUS: u8 = 7;

/// One line of a list fixture: a task row, or the indicator's line when it is visible.
#[derive(Clone, Copy)]
enum Piece {
    Row(&'static str),
    Line,
}

impl Piece {
    fn height(self) -> f32 {
        match self {
            Piece::Row(_) => ROW_H,
            Piece::Line => LINE_H,
        }
    }
}

/// The indicator: its props, and how it is drawn in place.
pub struct DropIndicator {
    p: DropindicatorProps,
}

impl DropIndicator {
    pub fn new(p: DropindicatorProps) -> Self {
        Self { p }
    }

    fn is_tab(&self) -> bool {
        self.p.target == "tab"
    }

    /// Draws the indicator in `rect` while it is visible, and tags its nodes: the line or bar is
    /// `indicator`, its dot is `dot` (absent on a tab). Hidden, it draws nothing: its `indicator`
    /// node is kept at zero size, so a test can still reach it and sees it as not visible.
    pub fn draw(&self, ui: &egui::Ui, look: Look, rect: Rect) {
        let shown = self.p.visible;
        let rect = if shown { rect } else { Rect::from_min_size(rect.min, vec2(0.0, 0.0)) };
        let t = look.t;
        let node = ui.interact(rect, ui.scope_id().with("indicator"), Sense::hover());
        node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &node, parts::INDICATOR);
        if !shown {
            return;
        }
        ui.painter().rect_filled(rect, CornerRadius::same(1), t.content_1);
        if self.is_tab() {
            return;
        }
        // The dot sits at the line's left end, centred on the line.
        let dot = Rect::from_center_size(pos2(rect.left() + DOT / 2.0, rect.center().y), vec2(DOT, DOT));
        let dot_node = ui.interact(dot, ui.scope_id().with("dot"), Sense::hover());
        dot_node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &dot_node, parts::DOT);
        ui.painter().circle_filled(dot.center(), DOT / 2.0, t.content_1);
    }

    /// The web harness's fixture, drawn around the indicator.
    fn fixture(&self, ui: &mut egui::Ui, look: Look) {
        if self.is_tab() {
            self.tab_fixture(ui, look);
        } else {
            self.list_fixture(ui, look);
        }
    }

    /// Two task rows: the line before or after the first for a row, or below both for the root.
    fn list_fixture(&self, ui: &mut egui::Ui, look: Look) {
        let (a, b) = ("Write release notes", "Review pull request");
        let root = self.p.target == "root";
        let after = self.p.edge == "after";
        // A hidden line is display:none on the web, so it takes no space and the rows close up.
        let line = self.p.visible.then_some(Piece::Line);
        let mut pieces: Vec<Piece> = Vec::new();
        if !root && !after {
            pieces.extend(line);
        }
        pieces.push(Piece::Row(a));
        if !root && after {
            pieces.extend(line);
        }
        pieces.push(Piece::Row(b));
        if root {
            pieces.extend(line);
        }
        let body: f32 = pieces.iter().map(|p| p.height()).sum();
        let total = 2.0 * PAD + body + GAP * (pieces.len() - 1) as f32;

        let (outer, _) = ui.allocate_exact_size(vec2(WIDTH, total), Sense::hover());
        let ui = &*ui;
        let mut y = outer.top() + PAD;
        for piece in pieces {
            let h = piece.height();
            let rect = Rect::from_min_size(pos2(outer.left() + PAD, y), vec2(WIDTH - 2.0 * PAD, h));
            match piece {
                Piece::Row(label) => draw_row(ui, look, rect, label),
                Piece::Line => self.draw(ui, look, rect),
            }
            y += h + GAP;
        }
        if !self.p.visible {
            // Hidden: only its zero-size node, where the line would sit.
            self.draw(ui, look, Rect::from_min_size(pos2(outer.left() + PAD, outer.top() + PAD), vec2(WIDTH - 2.0 * PAD, 0.0)));
        }
    }

    /// The web harness's tab strip: three tabs, with the bar on the divider after the first when visible.
    fn tab_fixture(&self, ui: &mut egui::Ui, look: Look) {
        let (outer, _) = ui.allocate_exact_size(vec2(WIDTH, TAB_H), Sense::hover());
        let ui = &*ui;
        let t = look.t;
        let painter = ui.painter();
        painter.rect(outer, CornerRadius::same(CONTROL_RADIUS), t.surface_1, Stroke::new(1.0, t.edge_1), StrokeKind::Inside);
        let cy = outer.center().y;
        // The strip's 1px border and its px-2 come first.
        let mut x = outer.left() + 1.0 + PAD;
        let mut gap = false;
        for (i, label) in ["Inbox", "Today", "Done"].into_iter().enumerate() {
            if i == 1 && self.p.visible {
                if gap {
                    x += GAP;
                }
                self.draw(ui, look, Rect::from_min_size(pos2(x, cy - BAR_H / 2.0), vec2(BAR_W, BAR_H)));
                x += BAR_W;
                gap = true;
            }
            if gap {
                x += GAP;
            }
            painter.text(pos2(x + PAD, cy), Align2::LEFT_CENTER, label, FontId::new(TEXT, weight(400)), t.content_2);
            x += label_width(painter, label) + 2.0 * PAD;
            gap = true;
        }
        if !self.p.visible {
            // Hidden: only its zero-size node, where the bar would sit.
            self.draw(ui, look, Rect::from_min_size(pos2(x, cy), vec2(BAR_W, BAR_H)));
        }
    }
}

/// A task row of the fixture: a field with a 1px edge and its label.
fn draw_row(ui: &egui::Ui, look: Look, rect: Rect, label: &str) {
    let t = look.t;
    let painter = ui.painter();
    painter.rect(rect, CornerRadius::same(FIELD_RADIUS), t.surface_2, Stroke::new(1.0, t.edge_1), StrokeKind::Inside);
    painter.text(pos2(rect.left() + PAD, rect.center().y), Align2::LEFT_CENTER, label, FontId::new(TEXT, weight(400)), t.content_1);
}

/// The width of a fixture label in its font (the colour does not change the width).
fn label_width(painter: &egui::Painter, label: &str) -> f32 {
    painter.layout_no_wrap(label.to_owned(), FontId::new(TEXT, weight(400)), Color32::WHITE).size().x
}

/// The desktop mount: the web harness's fixture drawn around the indicator.
pub struct DropIndicatorMount {
    widget: DropIndicator,
}

impl crate::Mounted for DropIndicatorMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, _emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        self.widget.fixture(ui, look);
    }
}

/// Builds the fixture and its indicator from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: DropindicatorProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(DropIndicatorMount { widget: DropIndicator::new(p) }))
}
