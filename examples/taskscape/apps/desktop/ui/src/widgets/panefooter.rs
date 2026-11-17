//! contract: omnicui/contracts/panefooter.contract.yaml (id: ui.panefooter). The stats footer of a task pane: "2 of 8 done · 25%" on the left and the caller's controls on the right, on a raised bar.
use crate::omnicui::footer::FooterProps;
use crate::omnicui::panefooter::{parts, PanefooterEvent, PanefooterProps};
use crate::widgets::footer::Footer;
use crate::{icon, part, rem, weight, Look, Mounted};
use egui::{pos2, vec2, Align2, CornerRadius, FontFamily, FontId, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// The web's stats text is text-[0.857rem] (12px) and its gap-1.5 is 5.25px. Its controls are ghost
/// footer buttons: 24px tall, px-2, a 14px icon and a 12px label at weight 500.
const STAT_SIZE: f32 = 12.0;
const STAT_GAP: f32 = 5.25;
const CONTROL_GAP: f32 = 8.0;
const BUTTON_H: f32 = rem(1.714);
const BUTTON_PAD: f32 = 8.0;
const ICON_SIZE: f32 = 14.0;
const ICON_GAP: f32 = 4.0;
const DIVIDER_H: f32 = rem(1.0);

/// Done percentage, rounded, or 0 when there is nothing to do (the web's percentDone).
pub fn percent_done(done: f64, total: f64) -> f64 {
    if total > 0.0 {
        (done / total * 100.0).round()
    } else {
        0.0
    }
}

/// A count as the web prints it: a whole number without a decimal point.
fn count(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

fn icon_font() -> FontId {
    FontId::new(ICON_SIZE, FontFamily::Name("Icons".into()))
}

/// The bar: a Footer raised one step with a top hairline, the stats on its left, and the caller's
/// controls (passed in by the mount) right-aligned.
pub struct PaneFooter {
    p: PanefooterProps,
    done: f64,
    total: f64,
    percent: Option<f64>,
    footer: Footer,
}

impl PaneFooter {
    pub fn new(p: PanefooterProps, done: f64, total: f64, percent: Option<f64>) -> Self {
        // The finished footer, with the web's tone (raised) and its top edge-2 hairline. The pane's
        // footer is not a window handle, so its drag region stays off.
        let bar: FooterProps = serde_json::from_value(serde_json::json!({
            "tone": "raised",
            "edge": "top",
            "density": "bar",
            "drag": "none",
        }))
        .expect("the footer's props");
        Self { p, done, total, percent, footer: Footer::new(bar) }
    }

    pub fn props(&self) -> &PanefooterProps {
        &self.p
    }

    /// One frame: draws the bar, the controls from `children` (which returns the left edge they take),
    /// and the stats. The bar raises no events of its own.
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        look: Look,
        children: impl FnOnce(&mut egui::Ui, Rect) -> f32,
    ) -> Vec<PanefooterEvent> {
        let (done, total) = (self.done, self.total);
        let pct = self.percent.unwrap_or_else(|| percent_done(done, total));
        let scope = ui.scope(|ui| {
            self.footer.ui(ui, look, |ui, content| {
                let controls_left = children(ui, content);
                stats(ui, look, content, controls_left, done, total, pct);
            })
        });
        // The part node covers the bar's whole rect, the way the web's footer element does.
        let response = ui.interact(scope.response.rect, ui.scope_id().with("panefooter"), Sense::hover());
        response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &response, parts::FOOTER);
        Vec::new()
    }
}

/// The stats: "N of M done" in content-1 at weight 500, a middle dot, and the percentage in content-2.
/// They are clipped short of the controls, as the web's truncate does.
fn stats(ui: &egui::Ui, look: Look, content: Rect, controls_left: f32, done: f64, total: f64, pct: f64) {
    let t = look.t;
    let clip = Rect::from_min_max(content.left_top(), pos2(controls_left - CONTROL_GAP, content.bottom()));
    let painter = ui.painter().with_clip_rect(clip);
    let y = content.center().y;
    let medium = FontId::new(STAT_SIZE, weight(500));
    let regular = FontId::new(STAT_SIZE, weight(400));
    let done_rect = painter.text(
        pos2(content.left(), y),
        Align2::LEFT_CENTER,
        format!("{} of {} done", count(done), count(total)),
        medium,
        t.content_1,
    );
    let dot = painter.text(pos2(done_rect.right() + STAT_GAP, y), Align2::LEFT_CENTER, "·", regular.clone(), t.content_2);
    painter.text(
        pos2(dot.right() + STAT_GAP, y),
        Align2::LEFT_CENTER,
        format!("{}%", count(pct)),
        regular,
        t.content_2,
    );
}

/// The harness's children, right-aligned in the bar: Sort, a vertical separator, Filter.
/// Returns the left edge they take.
fn controls(ui: &mut egui::Ui, look: Look, content: Rect) -> f32 {
    let items = [
        Item::Button { icon: "swap_vert", label: "Sort" },
        Item::Divider,
        Item::Button { icon: "filter_list", label: "Filter" },
    ];
    let widths: Vec<f32> = items.iter().map(|item| item.width(ui, look)).collect();
    let total: f32 = widths.iter().sum::<f32>() + CONTROL_GAP * (items.len() - 1) as f32;
    let left = content.right() - total;
    let mut x = left;
    for (item, w) in items.iter().zip(widths) {
        let h = match item {
            Item::Button { .. } => BUTTON_H,
            Item::Divider => DIVIDER_H,
        };
        let rect = Rect::from_center_size(pos2(x + w / 2.0, content.center().y), vec2(w, h));
        item.draw(ui, look, rect);
        x += w + CONTROL_GAP;
    }
    left
}

#[derive(Clone, Copy)]
enum Item {
    Button { icon: &'static str, label: &'static str },
    Divider,
}

impl Item {
    fn width(self, ui: &egui::Ui, look: Look) -> f32 {
        match self {
            Item::Button { icon: name, label } => {
                let painter = ui.painter();
                let glyph = painter
                    .layout_no_wrap(icon(name, ICON_SIZE).text().to_owned(), icon_font(), look.t.content_2)
                    .size()
                    .x;
                let text = painter
                    .layout_no_wrap(label.to_owned(), FontId::new(STAT_SIZE, weight(500)), look.t.content_2)
                    .size()
                    .x;
                2.0 * BUTTON_PAD + glyph + ICON_GAP + text
            }
            Item::Divider => 1.0,
        }
    }

    fn draw(self, ui: &mut egui::Ui, look: Look, rect: Rect) {
        match self {
            Item::Button { icon: name, label } => control_button(ui, look, rect, name, label),
            Item::Divider => {
                ui.painter().rect_filled(rect, CornerRadius::same(0), look.t.edge_2);
            }
        }
    }
}

/// A ghost footer button: an icon and a label, a wash on hover and a focus ring while focused.
fn control_button(ui: &mut egui::Ui, look: Look, rect: Rect, name: &str, label: &str) {
    let response = ui.interact(rect, ui.scope_id().with(("panefooter.control", label)), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    let t = look.t;
    let hovered = response.hovered();
    let ink = if hovered { t.content_1 } else { t.content_2 };
    let painter = ui.painter();
    if hovered {
        painter.rect_filled(rect, CornerRadius::same(7), t.wash_2);
    }
    if response.has_focus() {
        painter.rect_stroke(rect.expand(2.0), CornerRadius::same(9), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    let glyph = painter.text(
        pos2(rect.left() + BUTTON_PAD, rect.center().y),
        Align2::LEFT_CENTER,
        icon(name, ICON_SIZE).text().to_owned(),
        icon_font(),
        ink,
    );
    painter.text(
        pos2(glyph.right() + ICON_GAP, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::new(STAT_SIZE, weight(500)),
        ink,
    );
}

/// The bar mounted as the web harness renders it. The harness draws the Sort and Filter controls
/// inside the bar as its children; the stats come from the harness's done and total.
pub struct PaneFooterMounted {
    widget: PaneFooter,
}

impl Mounted for PaneFooterMounted {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.widget.ui(ui, look, |ui, content| controls(ui, look, content)) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the bar from a test's props. The generated props hold none of the harness's values, so
/// done, total and percent are read here, with the harness's defaults (done 2, total 8, percent derived).
pub fn mount(props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: PanefooterProps = serde_json::from_value(props.clone()).map_err(|e| format!("props: {e}"))?;
    let num = |k: &str| props.get(k).and_then(serde_json::Value::as_f64);
    let widget = PaneFooter::new(p, num("done").unwrap_or(2.0), num("total").unwrap_or(8.0), num("percent"));
    Ok(Box::new(PaneFooterMounted { widget }))
}
