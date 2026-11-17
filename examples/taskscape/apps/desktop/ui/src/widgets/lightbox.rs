//! contract: omnicui/contracts/lightbox.contract.yaml (id: ui.lightbox). A full-window viewer for one attachment: a stage that holds the preview, closed by Escape or a press on empty stage; the overlay presentation sits over the strong scrim, the window presentation is a centred dialog window.
use crate::omnicui::lightbox::{parts, LightboxEvent, LightboxProps};
use crate::omnicui::scrim::ScrimProps;
use crate::widgets::scrim::Scrim;
use crate::{part, rem, weight, Look, Tokens};
use egui::text::{LayoutJob, TextFormat};
use egui::{pos2, vec2, Area, Color32, CornerRadius, Frame, FontId, Galley, Key, Modifiers, Order, Painter, Rect, Role, Sense, Shadow, Stroke, WidgetInfo};
use std::sync::Arc;

/// The web's fade (--duration-fast), in seconds.
const FADE: f32 = 0.18;

pub struct Lightbox {
    p: LightboxProps,
    /// Shown. Escape or a press on the stage sets it false; the layer then fades out and is gone.
    open: bool,
}

impl Lightbox {
    pub fn new(p: LightboxProps) -> Self {
        let open = p.open;
        Self { p, open }
    }

    /// One frame: draws the lightbox with its default children (the attachment preview) and returns the
    /// events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<LightboxEvent> {
        let mut events = vec![];
        let ctx = ui.ctx().clone();

        // Escape closes while open. The key is consumed here, so the close is reported once.
        if self.open && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.open = false;
            events.push(LightboxEvent::Close { reason: "escape".to_owned() });
        }

        // The fade: `a` is 1 while open and falls to 0 over FADE after a close. At 0 nothing is drawn.
        let a = ctx.animate_bool_with_time(ui.scope_id().with("lightbox.fade"), self.open, FADE);
        if a <= 0.0 {
            return events;
        }

        let t = look.t;
        let screen = ctx.content_rect();
        let window = self.p.presentation == "window";
        // Overlay fills the window. Window is min(56rem, 100% - 2rem) by min(40rem, 100% - 2rem), centred.
        let frame_rect = if window {
            let size = vec2((screen.width() - rem(2.0)).clamp(0.0, rem(56.0)), (screen.height() - rem(2.0)).clamp(0.0, rem(40.0)));
            Rect::from_center_size(screen.center(), size)
        } else {
            screen
        };
        // The stage is px-16 pb-8 (4rem each side, 2rem at the bottom). The media wrapper fills what is left.
        let stage = frame_rect;
        let media = Rect::from_min_size(
            pos2(stage.left() + rem(4.0), stage.top()),
            vec2((stage.width() - rem(8.0)).max(0.0), (stage.height() - rem(2.0)).max(0.0)),
        );

        let open = self.open;
        let mut stage_pressed = false;
        let layer = Area::new(ui.scope_id().with("lightbox.layer"))
            .order(Order::Foreground)
            .movable(false)
            .fixed_pos(frame_rect.min);
        let shown = layer.show(&ctx, |ui| {
            let painter = ui.ctx().layer_painter(ui.layer_id());

            // Overlay: the strong scrim under the layer. Dismissal is the lightbox's own (Escape, stage),
            // so the scrim's press event is not used: outside presses do not close it, as on the web.
            if !window {
                let mut scrim = Scrim::new(ScrimProps { open: true, dim: "strong-blur".to_owned(), ..Default::default() });
                scrim.ui(ui, look);
            }

            // Window: the dialog box, with the web's panel radius, edge, surface and menu shadow.
            if window {
                let frame = Frame::new()
                    .fill(t.surface_2.gamma_multiply(a))
                    .stroke(Stroke::new(1.0, t.edge_1.gamma_multiply(a)))
                    .corner_radius(CornerRadius::same(10))
                    .shadow(shadow_menu(a));
                painter.add(frame.paint(frame_rect));
            }

            // The layer's node: a generic container at the layer's rect, named as the dialog.
            let node = ui.interact(frame_rect, ui.scope_id().with("lightbox"), Sense::hover());
            node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Attachment"));
            part(ui, &node, parts::LIGHTBOX);

            // The stage: a press on it, not on the media wrapper, closes the lightbox.
            let stage_resp = ui.interact(stage, ui.scope_id().with("stage"), Sense::click());
            stage_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &stage_resp, parts::STAGE);
            stage_pressed = stage_resp.clicked() && open;

            // The media wrapper: it takes presses so that they never reach the stage.
            let media_resp = ui.interact(media, ui.scope_id().with("media"), Sense::click());
            media_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &media_resp, parts::MEDIA);

            preview(ui.ctx(), &painter, media, t, a);
        });

        // Tab and the pointer stay inside the layer while it is shown.
        ctx.memory_mut(|m| m.set_modal_layer(shown.response.layer_id));

        if stage_pressed {
            self.open = false;
            events.push(LightboxEvent::Close { reason: "stage".to_owned() });
        }
        events
    }
}

/// The default children: an attachment preview card (the web harness's "Q3 plan.pdf"), centred in the media
/// wrapper. A card is max-w-[28rem], p-3, gap-2; its preview is aspect 4/3 with wash_2; its info is px-1, gap-0.5.
fn preview(ctx: &egui::Context, painter: &Painter, media: Rect, t: Tokens, a: f32) {
    let pad = 11.0;
    let gap = rem(0.5);
    let w = media.width().clamp(0.0, rem(28.0));
    let inner = (w - 2.0 * pad).max(0.0);
    let text_w = (inner - rem(0.5)).max(0.0);
    let prev_h = inner * 3.0 / 4.0;

    let name_color = t.content_1.gamma_multiply(a);
    let sub_color = t.content_2.gamma_multiply(a);
    let name = line(painter, "Q3 plan.pdf", 14.0, 600, name_color, text_w);
    let sub = line(painter, "PDF · 2.4 MB · Added today", 12.0, 400, sub_color, text_w);
    let (name_h, sub_h) = (name.size().y, sub.size().y);
    let info_h = name_h + rem(0.125) + sub_h;
    let h = pad + prev_h + gap + info_h + pad;
    let card = Rect::from_center_size(media.center(), vec2(w, h));
    let _ = ctx;

    painter.add(Frame::new().fill(t.surface_2.gamma_multiply(a)).corner_radius(CornerRadius::same(10)).shadow(shadow_menu(a)).paint(card));

    let x = card.left() + pad;
    let preview_rect = Rect::from_min_size(pos2(x, card.top() + pad), vec2(inner, prev_h));
    painter.rect_filled(preview_rect, CornerRadius::same(7), t.wash_2.gamma_multiply(a));
    let label_color = t.content_3.gamma_multiply(a);
    let label = line(painter, "Preview", 13.0, 400, label_color, inner);
    let label_pos = preview_rect.center() - label.size() / 2.0;
    painter.galley(label_pos, label, label_color);

    let info_x = x + rem(0.25);
    let name_y = preview_rect.bottom() + gap;
    painter.galley(pos2(info_x, name_y), name, name_color);
    painter.galley(pos2(info_x, name_y + name_h + rem(0.125)), sub, sub_color);
}

/// One line of text in Inter at a px size and a weight, truncated with an ellipsis when it is wider than max_w.
fn line(painter: &Painter, s: &str, px: f32, w: u16, color: Color32, max_w: f32) -> Arc<Galley> {
    let format = TextFormat { font_id: FontId::new(px, weight(w)), color, ..Default::default() };
    let mut job = LayoutJob::single_section(s.to_owned(), format);
    job.wrap.max_width = max_w.max(0.0);
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    painter.layout_job(job)
}

/// The web's shadow-menu, faded with the layer.
fn shadow_menu(a: f32) -> Shadow {
    Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha((64.0 * a) as u8) }
}

crate::mounted!(Lightbox, LightboxProps);
