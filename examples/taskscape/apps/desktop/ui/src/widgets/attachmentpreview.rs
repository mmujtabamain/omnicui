//! contract: omnicui/contracts/attachmentpreview.contract.yaml (id: ui.attachmentpreview). One attachment in the lightbox, drawn by its kind: image, video or audio poster, PDF poster, selectable text, or the fallback card when it fails.
use crate::omnicui::attachmentpreview::{parts, AttachmentpreviewEvent, AttachmentpreviewProps};
use crate::{icon, part, rem, text, weight, Look, Tokens};
use egui::{vec2, Align2, CornerRadius, FontId, Label, Rect, RichText, Role, ScrollArea, Sense, Stroke, StrokeKind, UiBuilder, WidgetInfo};
use std::io::Read;

/// The web's Tailwind widths at its 14px rem: max-w-md is 28rem, max-w-3xl 48rem, max-w-4xl 56rem.
const PANEL_MAX_W: f32 = rem(28.0);
const TEXT_MAX_W: f32 = rem(48.0);
const PDF_MAX_W: f32 = rem(56.0);
/// The image kind is w-[34.286rem] wide (480px) at 16:10.
const IMAGE_W: f32 = rem(34.286);
/// p-5 on the text block; px-8 py-10 and gap-3 on the audio panel.
const TEXT_PAD: f32 = rem(1.25);
const AUDIO_PAD_X: f32 = rem(2.0);
const AUDIO_PAD_Y: f32 = rem(2.5);
const AUDIO_GAP: f32 = rem(0.75);
/// The audio panel at its content: the padding, a 40px glyph, the name, Open, and two gaps.
const AUDIO_H: f32 = AUDIO_PAD_Y * 2.0 + 40.0 + AUDIO_GAP * 2.0 + 18.0 + 30.0;
/// The text kind is capped at this many characters.
const TEXT_LIMIT: usize = 262144;
/// rounded-(--radius-control) is 7px; rounded-(--radius-panel) is 10.5px.
const RADIUS_CONTROL: u8 = 7;
const RADIUS_PANEL: u8 = 11;

/// How the attachment resolved. The desktop reads it from disk, so this is known at once.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Status {
    Loaded,
    Error,
}

pub struct AttachmentPreview {
    p: AttachmentpreviewProps,
    /// None for a kind the widget does not know.
    outcome: Option<Status>,
    resolved: bool,
}

impl AttachmentPreview {
    pub fn new(p: AttachmentpreviewProps) -> Self {
        Self { p, outcome: None, resolved: false }
    }

    /// One frame: draws the attachment and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<AttachmentpreviewEvent> {
        let mut events = vec![];
        if !self.resolved {
            self.resolved = true;
            self.outcome = resolve(&self.p);
            let kind = self.p.kind.clone();
            match self.outcome {
                Some(Status::Loaded) => {
                    events.push(AttachmentpreviewEvent::Loaded { kind: kind.clone() });
                    // Desktop media plays in the OS's player (decision 26): autoPlay hands it over on mount.
                    if self.p.autoPlay && (kind == "video" || kind == "audio") {
                        open_in_os(&self.p.src);
                        events.push(AttachmentpreviewEvent::Playing { kind });
                    }
                }
                Some(Status::Error) => events.push(AttachmentpreviewEvent::Error { kind }),
                None => {}
            }
        }

        let t = look.t;
        let avail = ui.available_size();
        let size = vec2(finite(avail.x, IMAGE_W), finite(avail.y, IMAGE_W * 10.0 / 16.0));
        let (area, _) = ui.allocate_exact_size(size, Sense::hover());

        let p = &self.p;
        match (self.outcome, p.kind.as_str()) {
            (Some(Status::Error), kind) => fallback(ui, area, t, fallback_icon(kind), &p.name),
            (Some(Status::Loaded), "image") => image(ui, area, t, &p.name, &p.src),
            (Some(Status::Loaded), "video") => {
                let r = fit(area, 9.0 / 16.0, IMAGE_W);
                poster(ui, r, t, "movie", &p.name, parts::VIDEO, &p.src);
            }
            (Some(Status::Loaded), "audio") => audio_panel(ui, area, t, &p.name, &p.src),
            (Some(Status::Loaded), "pdf") => {
                let r = Rect::from_center_size(area.center(), vec2(area.width().min(PDF_MAX_W), area.height()));
                poster(ui, r, t, "picture_as_pdf", &p.name, parts::PDF, &p.src);
            }
            (Some(Status::Loaded), "text") => text_block(ui, area, t, &p.text),
            _ => {}
        }
        events
    }
}

crate::mounted!(AttachmentPreview, AttachmentpreviewProps);

/// Reads the attachment once. None for a kind the widget does not know.
fn resolve(p: &AttachmentpreviewProps) -> Option<Status> {
    let src = p.src.as_str();
    let ok = |b: bool| Some(if b { Status::Loaded } else { Status::Error });
    match p.kind.as_str() {
        "image" => ok(is_image(src)),
        "video" | "audio" | "pdf" => ok(is_file(src)),
        "text" => Some(Status::Loaded),
        _ => None,
    }
}

fn is_file(path: &str) -> bool {
    std::fs::metadata(path).map(|m| m.is_file() && m.len() > 0).unwrap_or(false)
}

/// The first bytes name the format: PNG, JPEG, GIF or WebP.
fn is_image(path: &str) -> bool {
    let mut head = Vec::new();
    if std::fs::File::open(path).and_then(|f| f.take(12).read_to_end(&mut head)).is_err() {
        return false;
    }
    head.starts_with(&[0x89, b'P', b'N', b'G'])
        || head.starts_with(&[0xFF, 0xD8, 0xFF])
        || head.starts_with(b"GIF8")
        || (head.len() >= 12 && head.starts_with(b"RIFF") && &head[8..12] == b"WEBP")
}

/// Hands a file to the OS's player or viewer. It does not wait for it; a failure leaves the poster in place.
fn open_in_os(path: &str) {
    #[cfg(target_os = "windows")]
    let spawned = std::process::Command::new("cmd").args(["/C", "start", "", path]).spawn();
    #[cfg(target_os = "macos")]
    let spawned = std::process::Command::new("open").arg(path).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let spawned = std::process::Command::new("xdg-open").arg(path).spawn();
    if let Ok(mut child) = spawned {
        std::thread::spawn(move || {
            let _ = child.wait();
        });
    }
}

fn fallback_icon(kind: &str) -> &'static str {
    match kind {
        "image" => "image_not_supported",
        "video" => "movie",
        "audio" => "audio_file",
        "pdf" => "picture_as_pdf",
        _ => "description",
    }
}

fn finite(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

/// The largest rect of a ratio (height over width) that fits the area and at most max_w wide, centred.
fn fit(area: Rect, ratio: f32, max_w: f32) -> Rect {
    let mut w = area.width().min(max_w);
    let mut h = w * ratio;
    if h > area.height() {
        h = area.height();
        w = h / ratio;
    }
    Rect::from_center_size(area.center(), vec2(w, h))
}

/// The text up to the limit, cut on a character boundary.
fn limit(s: &str) -> &str {
    match s.char_indices().nth(TEXT_LIMIT) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}

/// The image kind: the picture itself, as large as the stage allows at its own proportions (object-contain),
/// through the app's image loader as the attachment tiles do. While it loads, or when it can't be read, the
/// frame is drawn with its name and icon (it was only ever that frame, #12482).
fn image(ui: &mut egui::Ui, area: Rect, t: Tokens, name: &str, src: &str) {
    let uri = if src.contains("://") { src.to_owned() } else { format!("file://{src}") };
    if let Ok(egui::load::TexturePoll::Ready { texture }) = ui.ctx().try_load_texture(&uri, egui::TextureOptions::LINEAR, egui::SizeHint::default()) {
        let (tw, th) = (texture.size.x, texture.size.y);
        if tw > 0.0 && th > 0.0 {
            let s = (area.width() / tw).min(area.height() / th).min(1.0);
            let r = Rect::from_center_size(area.center(), vec2(tw * s, th * s));
            egui::Image::from_texture(texture).corner_radius(CornerRadius::same(RADIUS_CONTROL)).paint_at(ui, r);
            let resp = ui.interact(r, ui.scope_id().with(parts::IMAGE), Sense::hover());
            let label = name.to_owned();
            resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, label.clone()));
            part(ui, &resp, parts::IMAGE);
            return;
        }
    }
    let r = fit(area, 10.0 / 16.0, IMAGE_W);
    ui.painter().rect_filled(r, CornerRadius::same(RADIUS_CONTROL), t.surface_2);
    let resp = ui.interact(r, ui.scope_id().with(parts::IMAGE), Sense::hover());
    let label = name.to_owned();
    resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, label.clone()));
    part(ui, &resp, parts::IMAGE);
    let top = ((r.height() - 64.0) / 2.0).max(0.0);
    ui.scope_builder(UiBuilder::new().max_rect(r.shrink(14.0)), |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(top);
            ui.label(icon("image", 40.0).color(t.content_3));
            ui.label(text(name.to_owned(), rem(0.929), 400).color(t.content_3));
        });
    });
}

/// A video or PDF poster: the name, and Open, which plays it in the OS's player.
fn poster(ui: &mut egui::Ui, r: Rect, t: Tokens, icon_name: &str, name: &str, part_id: &'static str, src: &str) {
    ui.painter().rect_filled(r, CornerRadius::same(RADIUS_CONTROL), t.surface_2);
    let resp = ui.interact(r, ui.scope_id().with(part_id), Sense::hover());
    let label = name.to_owned();
    resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, label.clone()));
    part(ui, &resp, part_id);
    let top = ((r.height() - 108.0) / 2.0).max(0.0);
    ui.scope_builder(UiBuilder::new().max_rect(r.shrink(14.0)), |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(top);
            ui.label(icon(icon_name, 40.0).color(t.content_3));
            ui.add_space(8.0);
            ui.label(text(name.to_owned(), 13.0, 500).color(t.content_1));
            ui.add_space(12.0);
            open_button(ui, t, src);
        });
    });
}

/// The audio panel: surface-2, a 40px glyph, the name and Open.
fn audio_panel(ui: &mut egui::Ui, area: Rect, t: Tokens, name: &str, src: &str) {
    let r = Rect::from_center_size(area.center(), vec2(area.width().min(PANEL_MAX_W), area.height().min(AUDIO_H)));
    ui.painter().rect_filled(r, CornerRadius::same(RADIUS_PANEL), t.surface_2);
    let resp = ui.interact(r, ui.scope_id().with(parts::AUDIO), Sense::hover());
    let label = name.to_owned();
    resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, label.clone()));
    part(ui, &resp, parts::AUDIO);
    ui.scope_builder(UiBuilder::new().max_rect(r.shrink2(vec2(AUDIO_PAD_X, AUDIO_PAD_Y))), |ui| {
        ui.vertical_centered(|ui| {
            let glyph = ui.label(icon("audio_file", 40.0).color(t.content_3));
            let g = ui.interact(glyph.rect, ui.scope_id().with(parts::GLYPH), Sense::hover());
            g.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Audio"));
            part(ui, &g, parts::GLYPH);
            ui.add_space(AUDIO_GAP);
            ui.label(text(name.to_owned(), 13.0, 500).color(t.content_1));
            ui.add_space(AUDIO_GAP);
            open_button(ui, t, src);
        });
    });
}

/// The text kind: selectable monospace in a surface-2 block, scrolling when it is taller than the stage.
fn text_block(ui: &mut egui::Ui, area: Rect, t: Tokens, body: &str) {
    let body = limit(body);
    let w = area.width().min(TEXT_MAX_W);
    // The library installs no monospace face, so the block is set in Inter at the web's size.
    let font = FontId::new(rem(0.893), weight(400));
    let galley = ui.painter().layout(body.to_owned(), font.clone(), t.content_1, (w - 2.0 * TEXT_PAD).max(1.0));
    let h = (galley.size().y + 2.0 * TEXT_PAD).min(area.height());
    let r = Rect::from_center_size(area.center(), vec2(w, h));
    ui.painter().rect_filled(r, CornerRadius::same(RADIUS_CONTROL), t.surface_2);
    let resp = ui.interact(r, ui.scope_id().with(parts::TEXT), Sense::hover());
    let label = body.to_owned();
    resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, label.clone()));
    part(ui, &resp, parts::TEXT);
    let content = RichText::new(body.to_owned()).font(font).color(t.content_1);
    ui.scope_builder(UiBuilder::new().max_rect(r.shrink(TEXT_PAD)), |ui| {
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add(Label::new(content).selectable(true));
        });
    });
}

/// The EmptyState card, shown in place of a failed attachment.
fn fallback(ui: &mut egui::Ui, area: Rect, t: Tokens, icon_name: &str, name: &str) {
    let card = Rect::from_center_size(area.center(), vec2(area.width().min(PANEL_MAX_W), area.height().min(140.0)));
    ui.painter().rect_filled(card, CornerRadius::same(RADIUS_PANEL), t.surface_2);
    let resp = ui.interact(card, ui.scope_id().with(parts::FALLBACK), Sense::hover());
    resp.widget_info(|| WidgetInfo::labeled(Role::Status, true, "Preview unavailable"));
    part(ui, &resp, parts::FALLBACK);
    let inner = card.shrink(20.0);
    let top = ((inner.height() - 80.0) / 2.0).max(0.0);
    let name = name.to_owned();
    ui.scope_builder(UiBuilder::new().max_rect(inner), |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(top);
            ui.label(icon(icon_name, 40.0).color(t.content_3));
            ui.label(text("Preview unavailable", 14.0, 600).color(t.content_1));
            if !name.is_empty() {
                ui.label(text(name.clone(), 12.0, 400).color(t.content_3));
            }
        });
    });
}

/// The Open button: a surface-3 button with a focus ring. Activating it hands the file to the OS.
fn open_button(ui: &mut egui::Ui, t: Tokens, src: &str) {
    let (rect, resp) = ui.allocate_exact_size(vec2(88.0, 30.0), Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Open"));
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(RADIUS_CONTROL), t.surface_3);
    if resp.hovered() {
        painter.rect_filled(rect, CornerRadius::same(RADIUS_CONTROL), t.wash_1);
    }
    if resp.has_focus() {
        painter.rect_stroke(rect.expand(2.0), CornerRadius::same(9), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    painter.text(rect.center(), Align2::CENTER_CENTER, "Open", FontId::new(13.0, weight(600)), t.content_1);
    if resp.clicked() {
        open_in_os(src);
    }
}
