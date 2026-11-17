//! contract: omnicui/contracts/overlay.contract.yaml (id: ui.overlay). A modal layer shell: a floating layer over the window that closes on Escape and reports the dismissal; desktop draws no scrim.
use crate::omnicui::overlay::{parts, OverlayEvent, OverlayProps};
use crate::{part, rem, text, Look};
use egui::{pos2, vec2, Align, Align2, Area, Color32, CornerRadius, Frame, Key, Layout, Margin, Modifiers, Order, Response, Role, Sense, Shadow, Stroke, StrokeKind, TextEdit, WidgetInfo};

/// The layer shell: an Area over the window, focus held inside it while open, Escape captured (when
/// captureEscape is set), and the dismissal reported. Desktop draws no scrim.
pub struct Overlay {
    p: OverlayProps,
    /// Whether the layer was open last frame, to see the moment it opens.
    was_open: bool,
    /// The widget that held focus when the layer opened; focus returns to it on close.
    opener: Option<egui::Id>,
    /// Frames left in which the first control inside the layer takes focus on open.
    focus_tries: u8,
}

impl Overlay {
    pub fn new(p: OverlayProps) -> Self {
        Self { p, was_open: false, opener: None, focus_tries: 0 }
    }

    /// One frame with an empty layer.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<OverlayEvent> {
        self.show(ui, look, |_, _| {})
    }

    /// One frame: draws the layer with `content` inside it. `content` is told `focus_first` on the
    /// frames just after the layer opens, when its first control should take focus.
    pub fn show(&mut self, ui: &mut egui::Ui, look: Look, content: impl FnOnce(&mut egui::Ui, bool)) -> Vec<OverlayEvent> {
        let mut events = vec![];
        let ctx = ui.ctx().clone();
        if !self.p.open {
            self.was_open = false;
            return events;
        }
        if !self.was_open {
            self.was_open = true;
            // Without a trigger, focus returns to whatever held it when the layer opened.
            self.opener = ctx.memory(|m| m.focused());
            self.focus_tries = 3;
            ctx.request_repaint();
        }

        // Escape is captured only when captureEscape is set; otherwise it is left to the content.
        if self.p.captureEscape && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.close(&ctx, &mut events);
            return events;
        }

        let t = look.t;
        let screen = ctx.content_rect();
        let fill = self.p.placement == "fill";
        let top = self.p.placement == "top";
        let width = if fill {
            screen.width()
        } else if top {
            (screen.width() - rem(2.0)).min(rem(36.0))
        } else {
            (screen.width() - rem(2.0)).min(rem(28.0))
        };
        let max_h = if fill {
            screen.height()
        } else if top {
            screen.height() - rem(5.0)
        } else {
            screen.height() - rem(2.0)
        };
        let focus_first = self.focus_tries > 0;

        let area = Area::new(ui.scope_id().with("overlay.layer.area")).order(Order::Foreground).movable(false);
        let area = if fill {
            area.fixed_pos(screen.min)
        } else if top {
            area.anchor(Align2::CENTER_TOP, vec2(0.0, rem(4.0)))
        } else {
            area.anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        };

        let shown = area.show(&ctx, |ui| {
            let frame = if fill {
                Frame::new()
            } else {
                Frame::new()
                    .fill(t.surface_2)
                    .stroke(Stroke::new(1.0, t.edge_1))
                    .corner_radius(CornerRadius::same(10))
                    .shadow(Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(64) })
            };
            let rect = frame
                .show(ui, |ui| {
                    ui.set_width(width);
                    ui.set_max_height(max_h);
                    if fill {
                        ui.set_min_height(max_h);
                    }
                    content(ui, focus_first);
                })
                .response
                .rect;
            // The layer's node: a generic container at the layer's rect. It takes no focus of its own,
            // so Tab stays on the controls inside the layer.
            let node = ui.interact(rect, ui.scope_id().with("layer"), Sense::hover());
            node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &node, parts::LAYER);
            rect
        });
        let layer = shown.inner;
        // Tab and Shift+Tab stay inside the layer while it is open.
        ctx.memory_mut(|m| m.set_modal_layer(shown.response.layer_id));
        self.focus_tries = self.focus_tries.saturating_sub(1);
        if self.focus_tries > 0 {
            ctx.request_repaint();
        }

        // A press outside the layer dismisses it when dismissOn is outside.
        if self.p.dismissOn == "outside" && ctx.input(|i| i.pointer.any_pressed()) {
            let outside = ctx.input(|i| i.pointer.interact_pos()).is_some_and(|p| !layer.contains(p));
            if outside {
                self.close(&ctx, &mut events);
            }
        }

        events
    }

    /// Closes the layer, reports the dismissal, and returns focus to the opener.
    fn close(&mut self, ctx: &egui::Context, events: &mut Vec<OverlayEvent>) {
        self.p.open = false;
        self.was_open = false;
        self.focus_tries = 0;
        events.push(OverlayEvent::OpenChanged { open: false });
        events.push(OverlayEvent::Dismissed {});
        if let Some(id) = self.opener.take() {
            ctx.memory_mut(|m| m.request_focus(id));
        }
        ctx.request_repaint();
    }
}

/// The desktop mount: the layer with the fixtures the web harness draws beside it (an opener at the
/// top left and an outside button at the bottom left) and inside it (a title, a field, and a button).
/// The harness's open state lives here: the opener sets it, and openChanged clears it.
pub struct OverlayScreen {
    overlay: Overlay,
    open: bool,
    /// The field's text.
    name: String,
}

impl OverlayScreen {
    pub fn new(p: OverlayProps) -> Self {
        Self { open: p.open, overlay: Overlay::new(p), name: String::new() }
    }

    /// One frame: the fixtures, and the layer around its content.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<OverlayEvent> {
        let t = look.t;
        let ctx = ui.ctx().clone();
        let screen = ctx.content_rect();

        // The opener, top left, and the outside button, bottom left, beside the layer.
        let mut opened = false;
        Area::new(ui.scope_id().with("opener"))
            .order(Order::Middle)
            .fixed_pos(screen.min + vec2(rem(0.75), rem(0.75)))
            .show(&ctx, |ui| {
                let opener = outline_button(ui, t, "Open layer");
                part(ui, &opener, parts::OPENER);
                opened = opener.clicked();
            });
        let outside_at = pos2(screen.left() + rem(0.75), screen.bottom() - rem(0.75) - rem(2.0));
        Area::new(ui.scope_id().with("outside"))
            .order(Order::Middle)
            .fixed_pos(outside_at)
            .show(&ctx, |ui| {
                let outside = outline_button(ui, t, "Outside");
                part(ui, &outside, parts::OUTSIDE);
            });

        if opened {
            self.open = true;
        }
        self.overlay.p.open = self.open;
        let fill = self.overlay.p.placement == "fill";
        let name = &mut self.name;
        let events = self.overlay.show(ui, look, |ui, focus_first| {
            // The layer's content: a title, the field (first tabbable) and a button (last tabbable).
            let mut body = |ui: &mut egui::Ui| {
                Frame::new().inner_margin(Margin::same(11)).show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, rem(0.5));
                    let title = ui.label(text("Overlay", rem(1.071), 600).color(t.content_1));
                    title.widget_info(|| WidgetInfo::labeled(Role::Heading, true, "Overlay"));
                    part(ui, &title, parts::TITLE);

                    let field_frame = Frame::new()
                        .fill(t.surface_3)
                        .stroke(Stroke::new(1.0, t.edge_2))
                        .corner_radius(CornerRadius::same(5))
                        .inner_margin(Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.add(
                                TextEdit::singleline(&mut *name)
                                    .hint_text("Name")
                                    .frame(Frame::new())
                                    .font(egui::FontId::new(13.0, crate::weight(400)))
                                    .desired_width(f32::INFINITY),
                            )
                        });
                    let field = field_frame.inner;
                    if field.has_focus() {
                        ui.painter().rect_stroke(
                            field_frame.response.rect.expand(2.0),
                            CornerRadius::same(7),
                            Stroke::new(2.0, t.focus_1),
                            StrokeKind::Outside,
                        );
                    }
                    if focus_first && !field.has_focus() {
                        field.request_focus();
                    }
                    part(ui, &field, parts::FIELD);

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let done = primary_button(ui, t, "Done");
                        part(ui, &done, parts::ACTION);
                    });
                });
            };
            if fill {
                ui.vertical_centered(|ui| {
                    ui.set_max_width(rem(18.0));
                    body(ui);
                });
            } else {
                body(ui);
            }
        });

        for e in &events {
            if let OverlayEvent::OpenChanged { open } = e {
                self.open = *open;
            }
        }
        events
    }
}

impl crate::Mounted for OverlayScreen {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.ui(ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the screen from a test's props, as the macro does for the other widgets.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: OverlayProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(OverlayScreen::new(p)))
}

/// The web's outline button: a surface with an edge, its label in content_1.
fn outline_button(ui: &mut egui::Ui, t: crate::Tokens, label: &str) -> Response {
    ui.add(
        egui::Button::new(text(label, 13.0, 500).color(t.content_1))
            .fill(t.surface_3)
            .stroke(Stroke::new(1.0, t.edge_2))
            .corner_radius(CornerRadius::same(7))
            .min_size(vec2(0.0, rem(2.0))),
    )
}

/// The web's primary button: accent fill, its label in on_accent.
fn primary_button(ui: &mut egui::Ui, t: crate::Tokens, label: &str) -> Response {
    ui.add(
        egui::Button::new(text(label, 13.0, 600).color(t.on_accent))
            .fill(t.accent_500)
            .corner_radius(CornerRadius::same(7))
            .min_size(vec2(0.0, rem(2.0))),
    )
}
