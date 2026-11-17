//! contract: omnicui/contracts/icon.contract.yaml (id: ui.icon). One Material Symbols glyph from the ligature font,
//! decorative or labelled, in the web's named sizes, with a tone, an optional 32px tint badge, a groupHover reveal and a ccw90 turn.
use crate::omnicui::icon::{parts, IconEvent, IconProps};
use crate::{icon, part, rem, Look, Mounted};
use egui::{Color32, CornerRadius, Rect, Role, Sense, Shape, Vec2, WidgetInfo};
use std::f32::consts::FRAC_PI_2;

const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "display"];
const TONES: [&str; 8] = ["inherit", "primary", "secondary", "muted", "accent", "danger", "onMedia", "onMediaMuted"];
const TINTS: [&str; 3] = ["none", "accent", "danger"];
const REVEALS: [&str; 2] = ["always", "groupHover"];
const ROTATES: [&str; 2] = ["none", "ccw90"];

/// The glyph's px size for a named step (the web's rem, where 1rem = 14px).
fn glyph_px(size: &str) -> f32 {
    match size {
        "xs" => rem(0.857),
        "sm" => rem(1.0),
        "lg" => rem(1.286),
        "xl" => rem(1.429),
        "display" => rem(3.143),
        _ => rem(1.143),
    }
}

/// Turns a vector clockwise on screen by `a` radians, the convention of egui's `TextShape::angle`.
fn turn(v: Vec2, a: f32) -> Vec2 {
    let (s, c) = a.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

pub struct Icon {
    p: IconProps,
}

impl Icon {
    pub fn new(p: IconProps) -> Self {
        Self { p }
    }

    fn tinted(&self) -> bool {
        self.p.tint != "none"
    }

    /// The box the icon takes: the glyph's own, or the 32px badge when tinted. Empty when the name is empty.
    pub fn box_size(&self) -> Vec2 {
        if self.p.name.is_empty() {
            Vec2::ZERO
        } else if self.tinted() {
            Vec2::splat(rem(2.2857))
        } else {
            Vec2::splat(glyph_px(&self.p.size))
        }
    }

    /// The glyph colour: the tint's when tinted, else the tone's. `inherit` takes `inherited`, the parent's colour.
    fn colour(&self, look: Look, inherited: Color32) -> Color32 {
        let t = look.t;
        match (self.p.tint.as_str(), self.p.tone.as_str()) {
            ("accent", _) => t.accent_500,
            ("danger", _) => t.danger_500,
            (_, "inherit") => inherited,
            (_, "primary") => t.content_1,
            (_, "secondary") => t.content_2,
            (_, "accent") => t.accent_500,
            (_, "danger") => t.danger_500,
            (_, "onMedia") => Color32::WHITE,
            (_, "onMediaMuted") => Color32::from_white_alpha(153),
            _ => t.content_3,
        }
    }

    /// Paints the icon into `rect`: the badge's 32px box when tinted, else the glyph's own box.
    /// `inherited` is what `tone inherit` takes; `tile_hovered` drives `reveal groupHover`.
    /// Adds the badge and glyph parts.
    pub fn paint(&self, ui: &mut egui::Ui, rect: Rect, look: Look, inherited: Color32, tile_hovered: bool) {
        if self.p.name.is_empty() {
            return;
        }
        let t = look.t;
        let id = ui.scope_id().with("ui.icon");
        let px = if self.tinted() { glyph_px("lg") } else { glyph_px(&self.p.size) };
        let glyph_rect = if self.tinted() {
            let fill = if self.p.tint == "danger" { t.danger_100 } else { t.selection_1 };
            ui.painter().rect_filled(rect, CornerRadius::same(7), fill);
            let badge = ui.interact(rect, id.with("badge"), Sense::hover());
            badge.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &badge, parts::BADGE);
            Rect::from_center_size(rect.center(), Vec2::splat(px))
        } else {
            rect
        };

        let glyph = ui.interact(glyph_rect, id.with("glyph"), Sense::hover());
        let label = self.p.label.clone();
        glyph.widget_info(|| {
            if label.is_empty() {
                WidgetInfo::new(Role::GenericContainer)
            } else {
                WidgetInfo::labeled(Role::GenericContainer, true, &label)
            }
        });
        part(ui, &glyph, parts::GLYPH);

        // groupHover: the glyph fades in over 0.18 s while its tile is hovered.
        let alpha = if self.p.reveal == "groupHover" {
            ui.ctx().animate_bool_with_time(id.with("reveal"), tile_hovered, 0.18)
        } else {
            1.0
        };
        // ccw90: the glyph turns -90 degrees over 0.1 s.
        let turned = ui.ctx().animate_bool_with_time(id.with("rotate"), self.p.rotate == "ccw90", 0.1);
        let angle = -FRAC_PI_2 * turned;

        let text = icon(&self.p.name, px).text().to_owned();
        let font = egui::FontId::new(px, egui::FontFamily::Name("Icons".into()));
        let colour = self.colour(look, inherited).gamma_multiply(alpha);
        let galley = ui.painter().layout_no_wrap(text, font, colour);
        // The shape turns about its top-left corner, so move that corner to keep the glyph centred.
        let half = galley.size() * 0.5;
        let pos = glyph_rect.center() - turn(half, angle);
        let mut shape = egui::epaint::TextShape::new(pos, galley, colour);
        shape.angle = angle;
        ui.painter().add(Shape::Text(shape));
    }

    /// The widget as a plain child of a layout: it takes its box and paints the icon in it.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<IconEvent> {
        let (rect, response) = ui.allocate_exact_size(self.box_size(), Sense::hover());
        self.paint(ui, rect, look, look.t.content_2, response.hovered());
        vec![]
    }
}

/// The fixture the web harness draws: the icon in a centred tile (the part the groupHover test hovers),
/// on a dark 80% box with 8px padding for the media tones.
pub struct IconMounted {
    icon: Icon,
}

impl Mounted for IconMounted {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, _emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        let media = matches!(self.icon.p.tone.as_str(), "onMedia" | "onMediaMuted");
        let pad = if media { 8.0 } else { 0.0 };
        let size = self.icon.box_size() + Vec2::splat(2.0 * pad);
        let tile_rect = Rect::from_center_size(ui.max_rect().center(), size);
        let tile = ui.allocate_rect(tile_rect, Sense::hover());
        tile.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &tile, parts::TILE);
        if media {
            ui.painter().rect_filled(tile_rect, CornerRadius::same(7), Color32::from_black_alpha(204));
        }
        self.icon.paint(ui, tile_rect.shrink(pad), look, look.t.content_2, tile.hovered());
    }
}

fn allowed(field: &str, value: &str, options: &[&str]) -> Result<(), String> {
    if options.contains(&value) {
        Ok(())
    } else {
        Err(format!("icon {field} \"{value}\" is not one of {}", options.join(", ")))
    }
}

fn check(p: &IconProps) -> Result<(), String> {
    allowed("size", &p.size, &SIZES)?;
    allowed("tone", &p.tone, &TONES)?;
    allowed("tint", &p.tint, &TINTS)?;
    allowed("reveal", &p.reveal, &REVEALS)?;
    allowed("rotate", &p.rotate, &ROTATES)
}

/// Builds the widget from a test's props, refusing a value the contract does not allow.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: IconProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    check(&p)?;
    Ok(Box::new(IconMounted { icon: Icon::new(p) }))
}
