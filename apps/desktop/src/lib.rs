//! The desktop app's components. Each one is a plain egui widget with its own API: it is built from
//! its props, drawn with `ui`, and returns what happened as an event.
//!
//! Two things in here exist for omnicui, and nothing else does:
//! - every contract part is tagged with [`part`], which writes the part id into the widget's
//!   AccessKit node as its `author_id`, so that the omnicui egui target can find it;
//! - `omnicui/` holds the types generated from the contracts (props, events, part ids).
//!
//! Names, states and text are whatever the widgets report to AccessKit, the same tree a screen
//! reader reads.
pub mod counter;
pub mod switch;
pub mod omnicui;

use egui::{Color32, FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

/// DejaVu Sans, the one font every app draws with.
pub const FONT: &[u8] = include_bytes!("../../../shared/fonts/DejaVuSans.ttf");

/// Tags a widget as a contract part. Call it after the widget's `widget_info`.
pub fn part(ui: &egui::Ui, response: &egui::Response, id: &str) {
    ui.ctx().accesskit_node_builder(response.id, |node| node.set_author_id(id.to_owned()));
}

/// Deliberate defects, switched on with OMNICUI_DEFECTS=name,name…, so that every check failure
/// can be caused on purpose. With none set, the components are correct.
pub fn defect(name: &str) -> bool {
    std::env::var("OMNICUI_DEFECTS").unwrap_or_default().split(',').any(|d| d == name)
}

/// The app's look: DejaVu Sans 16px and these colours, the same on every app.
#[derive(Clone, Copy)]
pub struct Look {
    pub bg: Color32,
    pub fg: Color32,
    pub accent: Color32,
    pub off: Color32,
    pub thumb: Color32,
}

impl Look {
    pub fn of(scheme: &str) -> Self {
        if scheme == "dark" {
            Self { bg: hex(0x1b1d19), fg: hex(0xecebe6), accent: hex(0x8ab4f8), off: hex(0x6f746a), thumb: Color32::WHITE }
        } else {
            Self { bg: hex(0xfbfaf7), fg: hex(0x1d1f1b), accent: hex(0x1f5fbf), off: hex(0x8a8f84), thumb: Color32::WHITE }
        }
    }
}

fn hex(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

/// Makes DejaVu Sans egui's proportional font.
pub fn fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert("DejaVu Sans".into(), Arc::new(FontData::from_static(FONT)));
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "DejaVu Sans".into());
    ctx.set_fonts(fonts);
}

/// CSS ease-out, cubic-bezier(0, 0, 0.58, 1), as an egui easing function.
pub fn ease_out(t: f32) -> f32 {
    cubic_bezier(0.0, 0.0, 0.58, 1.0, t)
}

/// The value of a CSS cubic-bezier timing function at time fraction `t`.
pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    let curve = |p1: f32, p2: f32, s: f32| {
        let u = 1.0 - s;
        3.0 * u * u * s * p1 + 3.0 * u * s * s * p2 + s * s * s
    };
    let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if curve(x1, x2, mid) < t { lo = mid } else { hi = mid }
    }
    curve(y1, y2, (lo + hi) / 2.0)
}
