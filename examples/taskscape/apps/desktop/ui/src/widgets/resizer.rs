//! contract: omnicui/contracts/resizer.contract.yaml (id: ui.resizer). A drag handle between two columns: a 4px grab strip with a 1px hairline, a split divider (double-click, Home and Enter reset it) or a preview edge (drag and keys only).
use crate::omnicui::resizer::{parts, ResizerEvent, ResizerProps};
use crate::{part, rem, weight, Look, Tokens};
use egui::{Align2, CornerRadius, Key, Rect, Role, Sense, Stroke, StrokeKind, UiBuilder, Vec2, WidgetInfo};

/// The split divider's default split (the web harness's DEFAULT_SPLIT).
const DEFAULT_SPLIT: f64 = 0.5;
/// The web's `w-[0.2857rem]` grab strip.
const STRIP: f32 = rem(0.2857);
/// The web harness's fixture: `h-[10rem] w-[40rem]`.
const FIXTURE_W: f32 = rem(40.0);
const FIXTURE_H: f32 = rem(10.0);

fn clamp_in(v: f64, lo: f64, hi: f64) -> f64 {
    v.max(lo).min(hi)
}

pub struct Resizer {
    p: ResizerProps,
    /// The current value (splitRatio or previewWidth), always clamped to min and max.
    value: f64,
    /// The width of the row the strip sits in (the web's parent element); a split drag is a fraction of it.
    span: f32,
    /// The value and the pointer's x where the current drag began.
    drag: Option<(f64, f32)>,
    /// The time and place of the last primary release on the strip, for double-click.
    last_click: Option<(f64, egui::Pos2)>,
}

impl Resizer {
    pub fn new(p: ResizerProps) -> Self {
        let value = clamp_in(p.value, p.min, p.max);
        Self { p, value, span: rem(40.0), drag: None, last_click: None }
    }

    /// The current value: the split ratio, or the preview width in px.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Sets the value from outside (the app's state, which a key or another window may have changed).
    pub fn set_value(&mut self, v: f64) {
        if self.drag.is_none() {
            self.value = clamp_in(v, self.p.min, self.p.max);
        }
    }

    /// Sets the width of the row the strip sits in.
    pub fn set_span(&mut self, px: f32) {
        self.span = px;
    }

    fn split(&self) -> bool {
        self.p.reset == "double-click"
    }

    /// Value change per unit of rightward movement: the divider grows right, the preview edge shrinks right.
    fn sign(&self) -> f64 {
        if self.split() { 1.0 } else { -1.0 }
    }

    /// Value change per arrow press.
    fn step(&self) -> f64 {
        if self.split() { 0.01 } else { 1.0 }
    }

    fn commit(&mut self, next: f64, events: &mut Vec<ResizerEvent>) {
        self.value = clamp_in(next, self.p.min, self.p.max);
        events.push(ResizerEvent::Resized { value: self.value });
    }

    fn reset(&mut self, events: &mut Vec<ResizerEvent>) {
        self.value = clamp_in(DEFAULT_SPLIT, self.p.min, self.p.max);
        events.push(ResizerEvent::Reset {});
    }

    /// One frame: draws the strip and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<ResizerEvent> {
        let mut events = vec![];
        if !self.p.open {
            return events;
        }
        let t = look.t;
        let split = self.split();
        let h = ui.available_height();
        let (rect, response) = ui.allocate_exact_size(egui::vec2(STRIP, h), Sense::click_and_drag());
        let response = response.on_hover_cursor(egui::CursorIcon::ResizeColumn);
        let dragging = response.dragged();

        // A double-click restores the default split; a press never does. Two primary releases on the strip,
        // within 0.5 s and a few pixels apart, are a double-click too. They are read from the frame's raw
        // events, so the press does not have to have registered on this widget.
        let now = ui.input(|i| i.time);
        let releases: Vec<egui::Pos2> = ui.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: false, .. } => Some(*pos),
                    _ => None,
                })
                .collect()
        });
        let mut pair_double = false;
        for pos in releases.into_iter().filter(|p| rect.contains(*p)) {
            match self.last_click.take() {
                Some((t0, p0)) if now - t0 < 0.5 && pos.distance(p0) < 6.0 => pair_double = true,
                _ => self.last_click = Some((now, pos)),
            }
        }
        let raw_double = ui.input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary)) && ui.rect_contains_pointer(rect);
        if split && (response.double_clicked() || raw_double || pair_double) {
            self.reset(&mut events);
        }

        // Drag: the value moves from where the drag began, by the pointer's distance from its press.
        if dragging {
            if response.drag_started() {
                let x0 = ui.input(|i| i.pointer.press_origin()).map_or(rect.center().x, |p| p.x);
                self.drag = Some((self.value, x0));
            }
            if let (Some((start, x0)), Some(pos)) = (self.drag, response.interact_pointer_pos()) {
                if response.drag_delta() != Vec2::ZERO {
                    let dx = f64::from(pos.x - x0);
                    let moved = if split { dx / f64::from(self.span) } else { dx };
                    let next = start + self.sign() * moved;
                    self.commit(next, &mut events);
                }
            }
        } else {
            self.drag = None;
        }

        // Keys, while the strip has focus. Left and Right are the splitter's keys, so they do not move focus.
        if response.has_focus() {
            let id = response.id;
            ui.memory_mut(|m| m.set_focus_lock_filter(id, egui::EventFilter { horizontal_arrows: true, ..Default::default() }));
            let (left, right, home, end, enter) = ui.input(|i| {
                (
                    i.key_pressed(Key::ArrowLeft),
                    i.key_pressed(Key::ArrowRight),
                    i.key_pressed(Key::Home),
                    i.key_pressed(Key::End),
                    i.key_pressed(Key::Enter),
                )
            });
            let (sign, step, cur) = (self.sign(), self.step(), self.value);
            if right {
                self.commit(cur + sign * step, &mut events);
            }
            if left {
                self.commit(cur - sign * step, &mut events);
            }
            if home {
                if split {
                    self.reset(&mut events);
                } else {
                    let lo = self.p.min;
                    self.commit(lo, &mut events);
                }
            }
            if end {
                let hi = self.p.max;
                self.commit(hi, &mut events);
            }
            if enter && split {
                self.reset(&mut events);
            }
        }

        let (label, value) = (self.p.label.clone(), self.value);
        response.widget_info(|| WidgetInfo::slider(true, value, &label));
        part(ui, &response, parts::RESIZER);

        // The hairline: a node of its own, transparent at rest and edge-3 on hover and drag.
        let hot = dragging || ui.rect_contains_pointer(rect);
        let hot_t = ui.ctx().animate_bool_with_time(response.id.with("hot"), hot, 0.18);
        let hair = Rect::from_center_size(rect.center(), egui::vec2(1.0, rect.height()));
        let hairline = ui.interact(hair, response.id.with("hairline"), Sense::hover());
        hairline.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &hairline, parts::HAIRLINE);

        let painter = ui.painter();
        let strip_fill = if dragging { t.edge_3 } else { t.edge_2 };
        painter.rect_filled(rect, CornerRadius::ZERO, strip_fill.gamma_multiply(hot_t));
        painter.rect_filled(hair, CornerRadius::ZERO, t.edge_3.gamma_multiply(hot_t));
        // The focus ring is inset, as the web's ring-inset is.
        if response.has_focus() {
            painter.rect_stroke(rect, CornerRadius::ZERO, Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
        }
        events
    }
}

/// The desktop mount: the web harness's fixture (a pane, the resizer and an aside) drawn around the widget.
pub struct ResizerMount {
    p: ResizerProps,
    w: Resizer,
}

/// A title and its lines, padded 8px, at the pane's top-left (the web's DemoPane).
fn pane(p: &egui::Painter, rect: Rect, title: &str, lines: &[&str], t: Tokens) {
    let mut at = rect.min + egui::vec2(8.0, 8.0);
    let r = p.text(at, Align2::LEFT_TOP, title, egui::FontId::new(13.0, weight(500)), t.content_1);
    at.y = r.max.y + 4.0;
    for line in lines {
        let r = p.text(at, Align2::LEFT_TOP, *line, egui::FontId::new(12.0, weight(400)), t.content_3);
        at.y = r.max.y + 4.0;
    }
}

impl crate::Mounted for ResizerMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        let t = look.t;
        let split = self.p.reset == "double-click";
        let open = self.p.open;
        let (outer, _) = ui.allocate_exact_size(egui::vec2(FIXTURE_W, FIXTURE_H), Sense::hover());
        ui.painter().rect(outer, CornerRadius::same(10), t.surface_1, Stroke::new(1.0, t.edge_1), StrokeKind::Inside);
        let inner = outer.shrink(1.0);
        let strip = if open { STRIP } else { 0.0 };
        let total = inner.width();
        let value = self.w.value() as f32;
        // The split divider sets the left pane's share of the row; the preview edge sets the aside's width in px.
        let left_w = if split {
            total * value
        } else if open {
            total - strip - value
        } else {
            total
        };
        let left = Rect::from_min_size(inner.min, egui::vec2(left_w, inner.height()));
        let p = ui.painter().with_clip_rect(left);
        p.rect_filled(left, CornerRadius::ZERO, t.surface_2);
        pane(&p, left, "Tasks", &["Plan the sprint", "Review onboarding", "Ship the export"], t);

        if split || open {
            let aside_x = inner.min.x + left_w + strip;
            let aside = Rect::from_min_max(egui::pos2(aside_x, inner.min.y), inner.max);
            let p = ui.painter().with_clip_rect(aside);
            p.rect_filled(aside, CornerRadius::ZERO, t.surface_2);
            let title = if split { "Details" } else { "Preview" };
            pane(&p, aside, title, &["Updated 2 hours ago", "Owned by Ana"], t);
        }

        if open {
            let strip_rect = Rect::from_min_size(egui::pos2(inner.min.x + left_w, inner.min.y), egui::vec2(STRIP, inner.height()));
            self.w.set_span(outer.width());
            let w = &mut self.w;
            ui.scope_builder(
                UiBuilder::new().max_rect(strip_rect).layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    for e in w.ui(ui, look) {
                        let (name, fields) = e.to_json();
                        emit(name, fields);
                    }
                },
            );
        }
    }
}

/// Builds the widget from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: ResizerProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(ResizerMount { w: Resizer::new(p.clone()), p }))
}
