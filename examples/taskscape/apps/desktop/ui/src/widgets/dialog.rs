//! contract: omnicui/contracts/dialog.contract.yaml (id: ui.dialog). A modal card with an optional badge, a headline, optional detail copy, an optional text field (kind name), an optional footer left slot and a Cancel and Accept footer; it sits in an Overlay layer and draws no scrim.
use crate::omnicui::dialog::{parts, DialogEvent, DialogProps};
use crate::omnicui::overlay::OverlayProps;
use crate::widgets::overlay::Overlay;
use crate::{part, rem, text, weight, Look, Tokens};
use egui::{pos2, vec2, Color32, CornerRadius, FontId, Frame, Key, Margin, Modifiers, Rect, Response, Role, Sense, Shadow, Stroke, StrokeKind, TextEdit, UiBuilder, WidgetInfo};

/// The gap between the card's items (0.571rem), and the horizontal padding of a footer button.
const GAP: f32 = rem(0.571);
/// The gap between the footer's left slot and its buttons, and between Cancel and Accept (0.429rem).
const ROW_GAP: f32 = rem(0.429);

/// A press the card made this frame.
#[derive(Clone, Copy)]
enum Press {
    Cancel,
    Accept,
}

pub struct Dialog {
    p: DialogProps,
    /// Whether the card is shown. Cancel and Escape close an overlay card; a window card stays drawn,
    /// because its window is the host's to close.
    open: bool,
    /// The layer under the card: it holds focus inside the card and covers the window.
    overlay: Overlay,
    /// The name card's field text.
    field: String,
    /// The card's height last frame. The sheet is centred with it.
    card_h: f32,
    /// The widget that held focus at the end of the last frame.
    focused: Option<egui::Id>,
}

impl Dialog {
    pub fn new(p: DialogProps) -> Self {
        // The layer covers the window (placement fill) and the card is laid out here. Escape is read
        // by the card, and outside-press dismissal is undecided and untested in the contract, so the
        // layer dismisses on nothing and captures no key.
        let overlay_props: OverlayProps = serde_json::from_value(serde_json::json!({
            "open": p.open,
            "placement": "fill",
            "dismissOn": "none",
            "captureEscape": false,
        }))
        .expect("overlay props");
        let card_h = if p.kind == "confirm" { 122.0 } else { 141.0 };
        Self { open: p.open, overlay: Overlay::new(overlay_props), field: String::new(), card_h, focused: None, p }
    }

    /// One frame: draws the card over the window and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<DialogEvent> {
        let mut events = vec![];
        if !self.open {
            return events;
        }
        let t = look.t;
        let p = self.p.clone();
        // Escape closes the card from anywhere in it. It is consumed before the field sees it.
        let escape = ui.ctx().input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        // egui drops focus on Escape before the card runs, so the widget that had it takes it back.
        if escape {
            if let Some(id) = self.focused {
                ui.ctx().memory_mut(|m| m.request_focus(id));
            }
        }
        let mut press: Option<Press> = if escape { Some(Press::Cancel) } else { None };
        let field = &mut self.field;
        let card_h = &mut self.card_h;
        self.overlay.show(ui, look, |ui, focus_first| {
            card(ui, t, &p, field, card_h, focus_first, &mut press);
        });
        match press {
            Some(Press::Cancel) => {
                // An overlay card hides, as the web's owner does on openChanged. A window card stays.
                if p.presentation != "window" {
                    self.open = false;
                }
                events.push(DialogEvent::OpenChanged { open: false });
            }
            Some(Press::Accept) => events.push(DialogEvent::Accepted {}),
            None => {}
        }
        self.focused = ui.ctx().memory(|m| m.focused());
        events
    }
}

/// The card inside the Overlay layer: its geometry for the size and presentation, then its body.
fn card(
    ui: &mut egui::Ui,
    t: Tokens,
    p: &DialogProps,
    field: &mut String,
    card_h: &mut f32,
    focus_first: bool,
    press: &mut Option<Press>,
) {
    let screen = ui.ctx().content_rect();
    let fullscreen = p.size == "fullscreen";
    let palette = p.size == "palette";
    // Edge: the card fills the window (the window presentation, or the fullscreen size) and has no border.
    let edge = fullscreen || p.presentation == "window";
    // Width: 26rem for a sheet, 35rem for a palette, the window for fullscreen; never wider than the window.
    let bw = if fullscreen {
        screen.width()
    } else if palette {
        (screen.width() - rem(2.0)).min(rem(35.0))
    } else if edge {
        screen.width().min(rem(26.0))
    } else {
        (screen.width() - rem(2.0)).min(rem(26.0))
    };
    let bx = screen.center().x - bw / 2.0;
    // The palette hangs from 6.5rem and is at most 60vh; fullscreen starts at the top; a sheet is centred.
    let (by, max_h) = if palette {
        (screen.top() + rem(6.5), screen.height() * 0.6)
    } else if fullscreen {
        (screen.top(), screen.height())
    } else {
        let by = (screen.center().y - *card_h / 2.0).max(screen.top());
        (by, screen.bottom() - by)
    };
    let max_rect = Rect::from_min_size(pos2(bx, by), vec2(bw, max_h));
    if edge {
        ui.painter().rect_filled(screen, CornerRadius::ZERO, t.surface_2);
    }
    let enter = ui.input(|i| i.key_pressed(Key::Enter));
    let frame_rect = ui
        .scope_builder(UiBuilder::new().max_rect(max_rect), |ui| {
            // The card's base: surface 2, the menu elevation and a 0.75rem corner. Edge cards are bare.
            let frame = if edge {
                Frame::new()
            } else {
                Frame::new()
                    .fill(t.surface_2)
                    .stroke(Stroke::new(1.0, t.edge_1))
                    .corner_radius(CornerRadius::same(10))
                    .shadow(Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(64) })
            };
            frame
                .inner_margin(Margin { left: 15, right: 10, top: 15, bottom: 10 })
                .show(ui, |ui| {
                    ui.set_width(bw - 25.0);
                    ui.spacing_mut().item_spacing = vec2(0.0, GAP);
                    body(ui, t, p, field, focus_first, enter, press);
                })
                .response
                .rect
        })
        .inner;
    *card_h = frame_rect.height();
    let card_rect = if edge { screen } else { frame_rect };
    node(ui, card_rect, "card", None, parts::CARD);
}

/// The card's body: badge, headline, detail copy, the name field, and the footer with Cancel and Accept.
/// Cancel comes before Accept in the code, so Tab order is Cancel then Accept.
fn body(
    ui: &mut egui::Ui,
    t: Tokens,
    p: &DialogProps,
    field: &mut String,
    focus_first: bool,
    enter: bool,
    press: &mut Option<Press>,
) {
    let confirm = p.kind == "confirm";
    let destructive = p.tone == "destructive" || (p.tone.is_empty() && confirm);

    if !p.badge.is_empty() {
        let badge = Frame::new()
            .fill(t.wash_2)
            .corner_radius(CornerRadius::same(255))
            .inner_margin(Margin { left: 7, right: 7, top: 2, bottom: 2 })
            .show(ui, |ui| {
                ui.label(text(p.badge.clone(), 11.0, 500).color(t.content_2));
            });
        node(ui, badge.response.rect, "badge", Some(&p.badge), parts::BADGE);
    }

    let title = ui.label(text(p.title.clone(), 15.0, 600).color(t.content_1));
    title.widget_info(|| WidgetInfo::labeled(Role::Heading, true, &p.title));
    part(ui, &title, parts::TITLE);

    if !p.description.is_empty() {
        let description = ui.label(text(p.description.clone(), 13.0, 400).color(t.content_2));
        description.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &p.description));
        part(ui, &description, parts::DESCRIPTION);
    }

    if !confirm {
        let frame = Frame::new()
            .fill(t.surface_3)
            .stroke(Stroke::new(1.0, t.edge_2))
            .corner_radius(CornerRadius::same(5))
            .inner_margin(Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.add(
                    TextEdit::singleline(&mut *field)
                        .frame(Frame::new())
                        .font(FontId::new(13.0, weight(400)))
                        .text_color(t.content_1)
                        .desired_width(f32::INFINITY),
                )
            });
        let rect = frame.response.rect;
        let edit = frame.inner;
        if edit.has_focus() {
            ui.painter().rect_stroke(rect, CornerRadius::same(5), Stroke::new(1.0, t.accent_500), StrokeKind::Inside);
            ui.painter().rect_stroke(rect.expand(2.0), CornerRadius::same(7), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        if focus_first && !edit.has_focus() {
            edit.request_focus();
        }
        // Enter in the name field accepts. The field may surrender focus on Enter, so either counts.
        if enter && (edit.has_focus() || edit.lost_focus()) {
            *press = Some(Press::Accept);
        }
        part(ui, &edit, parts::FIELD);
    }

    let row = ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = ROW_GAP;
        if !p.instead.is_empty() {
            let instead = ui.label(text(p.instead.clone(), 12.0, 400).color(t.content_3));
            instead.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &p.instead));
            part(ui, &instead, parts::INSTEAD);
        }
        // The two buttons sit at the right: measure them, then leave the spare width to the left.
        let actions_w = action_w(ui, &p.cancelLabel) + action_w(ui, &p.acceptLabel) + ROW_GAP;
        let spare = ui.available_width() - actions_w;
        if spare > 0.0 {
            ui.add_space(spare);
        }
        let cancel = action(ui, t, &p.cancelLabel, t.surface_0, t.wash_2, t.content_1);
        part(ui, &cancel, parts::CANCEL);
        let (fill, hot) = if destructive { (t.danger_500, t.danger_600) } else { (t.accent_500, t.accent_600) };
        let accept = action(ui, t, &p.acceptLabel, fill, hot, t.on_accent);
        part(ui, &accept, parts::ACCEPT);
        (cancel, accept)
    });
    node(ui, row.response.rect, "footer", None, parts::FOOTER);
    let (cancel, accept) = row.inner;

    // A confirm card takes its initial focus on Cancel.
    if confirm && focus_first && !cancel.has_focus() {
        cancel.request_focus();
    }
    if cancel.clicked() {
        *press = Some(Press::Cancel);
    } else if accept.clicked() && !(confirm && enter) {
        // Enter on a focused Accept of a confirm does not accept; a press or Space does.
        *press = Some(Press::Accept);
    }
}

/// A footer button at the web's md size: 24.5px high, 8px side padding, a 13px medium label. It takes
/// focus, and a press, Enter or Space clicks it.
fn action(ui: &mut egui::Ui, t: Tokens, label: &str, fill: Color32, hot: Color32, fg: Color32) -> Response {
    let galley = ui.painter().layout_no_wrap(label.to_owned(), FontId::new(13.0, weight(500)), fg);
    let size = vec2(galley.size().x + 2.0 * GAP, rem(1.75));
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    let radius = CornerRadius::same(4);
    let painter = ui.painter();
    painter.rect_filled(rect, radius, if response.hovered() { hot } else { fill });
    painter.galley(rect.center() - galley.size() / 2.0, galley, fg);
    if response.has_focus() {
        painter.rect_stroke(rect, radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    response
}

/// The width a footer button with this label takes.
fn action_w(ui: &egui::Ui, label: &str) -> f32 {
    ui.painter().layout_no_wrap(label.to_owned(), FontId::new(13.0, weight(500)), Color32::WHITE).size().x + 2.0 * GAP
}

/// A part with no widget of its own (it is only drawn): a hover-sensed node tagged with the part id.
fn node(ui: &egui::Ui, rect: Rect, key: &str, label: Option<&str>, part_id: &str) {
    let r = ui.interact(rect, ui.scope_id().with(key), Sense::hover());
    r.widget_info(|| match label {
        Some(l) => WidgetInfo::labeled(Role::GenericContainer, true, l),
        None => WidgetInfo::new(Role::GenericContainer),
    });
    part(ui, &r, part_id);
}

crate::mounted!(Dialog, DialogProps);
