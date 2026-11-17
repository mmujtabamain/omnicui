//! Taskscape's desktop widget library, in egui. It holds the same widgets as the web's
//! packages/ui, each held to the same contract in omnicui/contracts:
//!
//! - every part a contract names is an AccessKit node whose `author_id` is the part id
//!   (`ui.button.label`), set with [`part`], so that the omnicui desktop target can find it;
//! - `omnicui/` holds the types generated from the contracts (props, events, part ids);
//! - colours come from [`tokens`] (the web's tokens.css, converted), sizes from [`rem`] (1rem = 14px),
//!   text from Inter at four weights and icons from the Material Symbols subset (see fonts/).
use egui::{FontData, FontDefinitions, FontFamily, FontId, Response, RichText};
use std::sync::Arc;

pub mod icons;
#[allow(non_snake_case)]
pub mod omnicui;
pub mod tokens;

pub use tokens::Tokens;

/// The widgets (src/widgets/), found by build.rs, and the registry from contract id to widget.
pub mod rich;

pub mod widgets {
    include!(concat!(env!("OUT_DIR"), "/widgets.rs"));
}

/// A widget or screen as omnicui's desktop target drives it: one frame at a time, reporting the
/// contract's events.
pub trait Mounted {
    /// Draws one frame. `emit` reports an event with its contract name and fields.
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value));
    /// Waiting for something outside the UI (the database): the target answers nothing until it isn't.
    fn busy(&self) -> bool {
        false
    }
    /// The OS asked the window to close (`do: close`).
    fn close(&mut self) {}
}

/// Implements [`Mounted`] for a widget whose `ui(&mut self, ui, look)` returns its events, and
/// gives its module the `mount` function the registry calls.
#[macro_export]
macro_rules! mounted {
    ($ty:ty, $props:ty) => {
        impl $crate::Mounted for $ty {
            fn frame(&mut self, ui: &mut egui::Ui, look: $crate::Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
                for e in self.ui(ui, look) {
                    let (name, fields) = e.to_json();
                    emit(name, fields);
                }
            }
        }
        /// Builds the widget from a test's props.
        pub fn mount(props: serde_json::Value) -> Result<Box<dyn $crate::Mounted>, String> {
            let p: $props = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
            Ok(Box::new(<$ty>::new(p)))
        }
    };
}

/// The look a widget draws with: the colour tokens of the light or dark scheme.
#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub t: Tokens,
    pub dark: bool,
}

impl Look {
    /// "light" or "dark", as a contract's colorSchemes names them.
    pub fn of(scheme: &str) -> Self {
        let dark = scheme == "dark";
        Self { t: if dark { tokens::DARK } else { tokens::LIGHT }, dark }
    }
}

/// Sizes are in the web's rem, where 1rem = 14px (decisions: "Sizes use 1rem = 14px").
pub const fn rem(r: f32) -> f32 {
    r * 14.0
}

/// Marks a widget's response as a contract part: AccessKit's `author_id` is the part id. Inside
/// [`as_part`], a part id the screen renamed is tagged with the screen's name instead.
pub fn part(ui: &egui::Ui, response: &Response, id: &str) {
    // Renames chain: a widget's part renamed by its screen (ui.tasktreerow.row -> app.tasklist.row) and that
    // screen's part renamed by the window around it (app.tasklist.row -> app.mainwindow.row).
    let map = ui.ctx().data(|d| d.get_temp::<Vec<(String, String)>>(part_map_id())).unwrap_or_default();
    let mut id = id.to_owned();
    for _ in 0..4 {
        match map.iter().rev().find(|(from, _)| *from == id) {
            Some((_, to)) if *to != id => id = to.clone(),
            _ => break,
        }
    }
    ui.ctx().accesskit_node_builder(response.id, |node| node.set_author_id(id));
}

fn part_map_id() -> egui::Id {
    egui::Id::new("taskscape-part-renames")
}

/// Draws a library widget under a screen's own part names: while `f` runs, a part the widget tags
/// as `from` (`ui.button.button`) is tagged `to` (`app.toasthost.undo`) instead. This is the desktop
/// form of the web's space-separated `data-part` tokens.
///
/// ```ignore
/// as_part(ui, &[(omnicui::button::parts::BUTTON, parts::UNDO)], |ui| self.undo.ui(ui, look));
/// ```
pub fn as_part<R>(ui: &mut egui::Ui, renames: &[(&str, &str)], f: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let id = part_map_id();
    let before = ui.ctx().data(|d| d.get_temp::<Vec<(String, String)>>(id)).unwrap_or_default();
    let mut map = before.clone();
    map.extend(renames.iter().map(|(a, b)| ((*a).to_owned(), (*b).to_owned())));
    ui.ctx().data_mut(|d| d.insert_temp(id, map));
    let out = f(ui);
    ui.ctx().data_mut(|d| d.insert_temp(id, before));
    out
}

/// Inter's weights as font families: `weight(600)` is SemiBold.
pub fn weight(w: u16) -> FontFamily {
    FontFamily::Name(match w {
        0..=449 => "Inter",
        450..=549 => "Inter Medium",
        550..=649 => "Inter SemiBold",
        _ => "Inter Bold",
    }.into())
}

/// Text in Inter at a size in px and a weight.
pub fn text(s: impl Into<String>, size: f32, w: u16) -> RichText {
    RichText::new(s).font(FontId::new(size, weight(w)))
}

/// An icon by its Material Symbols name ("check"), at a size in px. An unknown name draws nothing.
pub fn icon(name: &str, size: f32) -> RichText {
    let glyph = icons::ICONS.iter().find(|(n, _)| *n == name).map(|(_, c)| c.to_string()).unwrap_or_default();
    RichText::new(glyph).font(FontId::new(size, FontFamily::Name("Icons".into())))
}

/// Installs Inter (four weights) as the proportional font and the icon font as the "Icons" family,
/// and the image loaders (attachment thumbnails, the lightbox).
pub fn fonts(ctx: &egui::Context) {
    egui_extras::install_image_loaders(ctx);
    let mut fonts = FontDefinitions::default();
    let faces: [(&str, &'static [u8]); 5] = [
        ("Inter", include_bytes!("../fonts/Inter-Regular.ttf")),
        ("Inter Medium", include_bytes!("../fonts/Inter-Medium.ttf")),
        ("Inter SemiBold", include_bytes!("../fonts/Inter-SemiBold.ttf")),
        ("Inter Bold", include_bytes!("../fonts/Inter-Bold.ttf")),
        ("Icons", include_bytes!("../fonts/MaterialSymbols.ttf")),
    ];
    for (name, bytes) in faces {
        fonts.font_data.insert(name.into(), Arc::new(FontData::from_static(bytes)));
        // Each face is a family of its own, falling back to egui's defaults for glyphs it lacks.
        let mut chain = vec![name.to_owned()];
        chain.extend(fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default());
        fonts.families.insert(FontFamily::Name(name.into()), chain);
    }
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "Inter".into());
    // The monospace family falls back to Inter: without egui's bundled fonts it would be empty, and egui
    // panics drawing any monospace text (its own debug warnings among them).
    fonts.families.entry(FontFamily::Monospace).or_default().push("Inter".into());
    ctx.set_fonts(fonts);
}

/// An eyebrow label (APPEARANCE, NOTES, a group's name): capitals at 10.5 px, weight 600, with the
/// reference's 0.12 em tracking (its SectionHeader: `text-[10.5px] tracking-[0.12em] uppercase`).
pub fn eyebrow(painter: &egui::Painter, label: &str, color: egui::Color32) -> std::sync::Arc<egui::Galley> {
    let px = 10.5;
    let format = egui::TextFormat { font_id: egui::FontId::new(px, weight(600)), color, extra_letter_spacing: 0.12 * px, ..Default::default() };
    painter.layout_job(egui::text::LayoutJob::single_section(label.to_uppercase(), format))
}
