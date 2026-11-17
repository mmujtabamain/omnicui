//! contract: omnicui/contracts/attachmenttile.contract.yaml (id: ui.attachmenttile). One grid tile for one attachment: a thumbnail for an image or a link glyph for a remote link; a click, Enter or Space opens it, and a right-click, the context-menu key or Shift+F10 asks for its menu.
use crate::omnicui::attachmenttile::{parts, AttachmenttileEvent, AttachmenttileProps};
use crate::{icon, part, rem, Look};
use egui::{pos2, vec2, Align2, CornerRadius, FontFamily, FontId, Key, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// What a drag carries: the attachment's name (the drop target reads it from the payload).
#[derive(Clone)]
pub struct AttachmentDrag {
    pub name: String,
}

pub struct AttachmentTile {
    p: AttachmenttileProps,
    /// The tile's side; a grid that fills its row sets it (the reference's tiles are at least 72 px and
    /// share the row's width).
    pub side: f32,
}

impl AttachmentTile {
    pub fn new(p: AttachmenttileProps) -> Self {
        Self { p, side: rem(6.714) }
    }

    fn is_link(&self) -> bool {
        self.p.kind == "link"
    }

    /// The kind as the events name it: "link" or "image".
    fn kind(&self) -> &'static str {
        if self.is_link() {
            "link"
        } else {
            "image"
        }
    }

    /// One frame: draws the tile and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<AttachmenttileEvent> {
        let mut events = vec![];
        let t = look.t;
        let side = self.side;
        let radius = CornerRadius::same(7);
        let sense = if self.p.draggable { Sense::click_and_drag() } else { Sense::click() };
        let (rect, response) = ui.allocate_exact_size(vec2(side, side), sense);
        let kind = self.kind();

        // A click opens it. Enter and Space on the focused tile are turned into a click by egui.
        if response.clicked() {
            events.push(AttachmenttileEvent::Opened { kind: kind.to_owned() });
        }

        // The menu: a right-click, or the context-menu key or Shift+F10 while the tile has focus.
        let focused = response.has_focus();
        // egui's Key has no context-menu key, so only Shift+F10 is read from the keyboard here. The
        // context-menu key reaches egui as the accessibility request to show the tile's menu instead.
        let shift_f10 = focused && ui.input(|i| i.modifiers.shift && i.key_pressed(Key::F10));
        let target = response.id.accesskit_id();
        let menu_request = ui.input(|i| {
            i.events.iter().any(|e| {
                matches!(e, egui::Event::AccessKitActionRequest(r)
                    if matches!(r.action, egui::accesskit::Action::ShowContextMenu) && r.target_node == target)
            })
        });
        if response.secondary_clicked() || shift_f10 || menu_request {
            events.push(AttachmenttileEvent::ContextMenuOpened { kind: kind.to_owned() });
        }

        // A drag carries the attachment into a note.
        if self.p.draggable && response.drag_started() {
            egui::DragAndDrop::set_payload(ui.ctx(), AttachmentDrag { name: self.p.name.clone() });
            events.push(AttachmenttileEvent::DragStarted { kind: kind.to_owned() });
        }

        let name = self.p.name.as_str();
        response.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
        part(ui, &response, parts::TILE);

        let hovered = response.hovered();
        let edge = if hovered { t.edge_3 } else { t.edge_2 };
        ui.painter().rect(rect, radius, t.surface_3, Stroke::new(1.0, edge), StrokeKind::Inside);
        // The thumbnail sits inside the 1px border (the web's padding box), clipped to its inner corners.
        let inner = rect.shrink(1.0);

        if self.is_link() {
            // The link glyph, 22px, decorative.
            let glyph_rect = Rect::from_center_size(rect.center(), vec2(22.0, 22.0));
            let g = ui.interact(glyph_rect, response.id.with("glyph"), Sense::hover());
            g.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &g, parts::GLYPH);
            let glyph = icon("link", 22.0);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                glyph.text(),
                FontId::new(22.0, FontFamily::Name("Icons".into())),
                t.content_3,
            );
        } else {
            // The thumbnail fills the tile (object-cover), drawn once it has loaded.
            let th = ui.interact(inner, response.id.with("thumb"), Sense::hover());
            th.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &th, parts::THUMB);
            self.paint_thumb(ui, inner, CornerRadius::same(6));
        }

        // The open_in_full overlay while the tile is hovered. It also shows while the tile has keyboard
        // focus, so that Enter and Space leave the same visible state as a press (which hovers the tile).
        if hovered || focused {
            let circle = Rect::from_center_size(rect.center(), vec2(rem(1.714), rem(1.714)));
            let o = ui.interact(circle, response.id.with("overlay"), Sense::hover());
            o.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &o, parts::OVERLAY);
            let painter = ui.painter();
            painter.circle_filled(circle.center(), circle.width() / 2.0, t.surface_2);
            let glyph = icon("open_in_full", rem(1.143));
            painter.text(
                circle.center(),
                Align2::CENTER_CENTER,
                glyph.text(),
                FontId::new(rem(1.143), FontFamily::Name("Icons".into())),
                t.content_1,
            );
        }

        // The focus ring: 2px of focus_1 just outside the border (ring-2 is a box shadow).
        if focused {
            ui.painter().rect_stroke(rect.expand(1.0), CornerRadius::same(8), Stroke::new(2.0, t.focus_1), StrokeKind::Middle);
        }
        events
    }

    /// Draws the thumbnail covering the tile, cropped from its centre (object-cover).
    fn paint_thumb(&self, ui: &egui::Ui, rect: Rect, radius: CornerRadius) {
        let src = &self.p.thumbnailSrc;
        if src.is_empty() {
            return;
        }
        let uri = if src.contains("://") { src.clone() } else { format!("file://{src}") };
        match ui.ctx().try_load_texture(&uri, egui::TextureOptions::LINEAR, egui::SizeHint::default()) {
            Ok(egui::load::TexturePoll::Ready { texture }) => {
                let (tw, th) = (texture.size.x, texture.size.y);
                if tw <= 0.0 || th <= 0.0 {
                    return;
                }
                let s = (rect.width() / tw).max(rect.height() / th);
                let uv = Rect::from_center_size(pos2(0.5, 0.5), vec2(rect.width() / (tw * s), rect.height() / (th * s)));
                egui::Image::from_texture(texture).uv(uv).corner_radius(radius).paint_at(ui, rect);
            }
            // The loader asks for a frame itself once the image is in; asking here every frame would keep a
            // headless run from ever settling.
            Ok(egui::load::TexturePoll::Pending { .. }) => {}
            Err(_) => {}
        }
    }
}

crate::mounted!(AttachmentTile, AttachmenttileProps);
