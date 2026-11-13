//! The design tokens (DESIGN.md, "Colour"), generated from the sRGB hex values there, in a light
//! and a dark variant. Eight-digit values carry alpha and are drawn over the surface beneath.

use gpui::{rgba, Hsla, Rgba};

#[derive(Clone, Copy)]
pub struct Theme {
    pub dark: bool,
    pub surface_0: Hsla,
    pub surface_1: Hsla,
    pub surface_2: Hsla,
    pub surface_3: Hsla,
    pub edge_1: Hsla,
    pub edge_2: Hsla,
    pub edge_3: Hsla,
    pub content_1: Hsla,
    pub content_2: Hsla,
    pub content_3: Hsla,
    pub accent_500: Hsla,
    pub accent_600: Hsla,
    pub accent_700: Hsla,
    pub on_accent: Hsla,
    pub selection_1: Hsla,
    pub focus_1: Hsla,
    pub wash_1: Hsla,
    pub wash_2: Hsla,
    pub danger_500: Hsla,
    pub danger_600: Hsla,
    pub danger_100: Hsla,
    pub success_500: Hsla,
    pub done_lamp_1: Hsla,
    /// The elevation shadows' colour (`oklch(0.2 0.02 262)`).
    pub shadow: Hsla,
}

fn c(hex: u32) -> Hsla {
    Hsla::from(rgba((hex << 8) | 0xff))
}
fn ca(hex: u32) -> Hsla {
    Hsla::from(rgba(hex))
}

pub const LIGHT_WINDOW: u32 = 0xf3f4f6;
pub const DARK_WINDOW: u32 = 0x161719;

impl Theme {
    pub fn light() -> Theme {
        Theme {
            dark: false,
            surface_0: c(0xebecee),
            surface_1: c(0xf3f4f6),
            surface_2: c(0xfdfdfe),
            surface_3: c(0xffffff),
            edge_1: ca(0x2d333d13),
            edge_2: ca(0x2d333d24),
            edge_3: ca(0x2d333d4c),
            content_1: c(0x21242a),
            content_2: c(0x545860),
            content_3: c(0x82868e),
            accent_500: c(0x007df3),
            accent_600: c(0x006ce7),
            accent_700: c(0x005bd6),
            on_accent: c(0xfafcff),
            selection_1: ca(0x007df31a),
            focus_1: ca(0x007df380),
            wash_1: ca(0x2d333d0d),
            wash_2: ca(0x2d333d17),
            danger_500: c(0xcc2827),
            danger_600: c(0xbb0916),
            danger_100: ca(0xcc28271a),
            success_500: c(0x319751),
            done_lamp_1: c(0x4c88d3),
            shadow: c(0x14171c),
        }
    }

    pub fn dark() -> Theme {
        Theme {
            dark: true,
            surface_0: c(0x0f1012),
            surface_1: c(0x161719),
            surface_2: c(0x1e1f22),
            surface_3: c(0x292b2e),
            edge_1: ca(0xf5f9ff0f),
            edge_2: ca(0xf5f9ff1c),
            edge_3: ca(0xf5f9ff3d),
            content_1: c(0xe9ebee),
            content_2: c(0xa8abb0),
            content_3: c(0x74787e),
            accent_500: c(0x3a9bfd),
            accent_600: c(0x53abff),
            accent_700: c(0x268beb),
            on_accent: c(0xfafcff),
            selection_1: ca(0x3a9bfd29),
            focus_1: ca(0x3a9bfd99),
            wash_1: ca(0xf5f9ff0b),
            wash_2: ca(0xf5f9ff14),
            danger_500: c(0xf75d59),
            danger_600: c(0xff6f69),
            danger_100: ca(0xf75d5926),
            success_500: c(0x62bb78),
            done_lamp_1: c(0x2f82d6),
            shadow: c(0x14171c),
        }
    }

    pub fn of(dark: bool) -> Theme {
        if dark {
            Theme::dark()
        } else {
            Theme::light()
        }
    }

    pub fn shadow_alpha(&self, a: f32) -> Hsla {
        let mut s = self.shadow;
        s.a = a;
        s
    }

    /// The native window's background before the first frame (`main/src-tauri/src/lib.rs`).
    pub fn window_background(&self) -> Rgba {
        rgba(((if self.dark { DARK_WINDOW } else { LIGHT_WINDOW }) << 8) | 0xff)
    }
}

/// DESIGN.md "Spacing, radii": radii in px at the 14 px root.
pub const R_FIELD: f32 = 4.4;
pub const R_CONTROL: f32 = 7.0;
pub const R_PANEL: f32 = 10.5;
