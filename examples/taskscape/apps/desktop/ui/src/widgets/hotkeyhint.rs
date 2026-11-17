//! contract: omnicui/contracts/hotkeyhint.contract.yaml (id: ui.hotkeyhint). Display-only glyph sequence for one accelerator: the platform modifier glyphs, then the key, edge to edge.
use crate::omnicui::hotkeyhint::{parts, HotkeyhintEvent, HotkeyhintProps};
use crate::{icon, part, rem, weight, Look};
use egui::{FontFamily, FontId, Role, Sense, WidgetInfo};

/// The glyph map to use. "" resolves from the host OS (Linux and other hosts take `Other`).
#[derive(Clone, Copy)]
enum Platform {
    Macos,
    Windows,
    Other,
}

fn resolve_platform(p: &str) -> Platform {
    match p {
        "macos" => Platform::Macos,
        "windows" => Platform::Windows,
        "other" => Platform::Other,
        _ if cfg!(target_os = "macos") => Platform::Macos,
        _ if cfg!(target_os = "windows") => Platform::Windows,
        _ => Platform::Other,
    }
}

/// One piece of the sequence: a Material Symbols glyph, or a word in Inter where no glyph is asserted.
struct Token {
    glyph: Option<&'static str>,
    text: String,
    /// The spoken equivalent, which is the part's name.
    label: String,
}

impl Token {
    fn glyph(name: &'static str, label: &str) -> Self {
        Self { glyph: Some(name), text: String::new(), label: label.to_owned() }
    }

    fn word(text: &str, label: &str) -> Self {
        Self { glyph: None, text: text.to_owned(), label: label.to_owned() }
    }
}

/// A modifier of the accelerator, on the platform's glyph map (the web's MODIFIERS).
fn modifier(name: &str, platform: Platform) -> Token {
    match (platform, name) {
        (Platform::Macos, "Meta") => Token::glyph("keyboard_command_key", "Command"),
        (Platform::Macos, "Alt") => Token::glyph("keyboard_option_key", "Option"),
        (Platform::Macos, "Control") => Token::glyph("keyboard_control_key", "Control"),
        (Platform::Macos, "Shift") => Token::glyph("arrow_upward", "Shift"),
        (Platform::Windows, "Meta" | "Control") => Token::glyph("keyboard_control_key", "Control"),
        (Platform::Windows, "Alt") => Token::word("Alt", "Alt"),
        (Platform::Windows, "Shift") => Token::glyph("arrow_upward", "Shift"),
        (Platform::Other, "Meta") => Token::word("Super", "Super"),
        (Platform::Other, "Alt") => Token::word("Alt", "Alt"),
        (Platform::Other, "Control") => Token::word("Ctrl", "Control"),
        (Platform::Other, "Shift") => Token::word("Shift", "Shift"),
        _ => Token::word(name, name),
    }
}

/// The key, last (the web's KEYS map). A single character is upper-cased.
fn key_token(key: &str) -> Token {
    match key {
        "Enter" | "Return" => Token::glyph("keyboard_return", "Return"),
        "Backspace" => Token::glyph("backspace", "Backspace"),
        "ArrowUp" => Token::glyph("arrow_upward", "Up"),
        "ArrowLeft" => Token::glyph("arrow_back", "Left"),
        "Comma" => Token::word(",", "Comma"),
        "Backslash" => Token::word("\\", "Backslash"),
        "Minus" => Token::word("-", "Minus"),
        _ => {
            let text = if key.chars().count() == 1 { key.to_uppercase() } else { key.to_owned() };
            Token::word(&text, &text)
        }
    }
}

pub struct Hotkeyhint {
    p: HotkeyhintProps,
}

impl Hotkeyhint {
    pub fn new(p: HotkeyhintProps) -> Self {
        Self { p }
    }

    /// One frame: draws the accelerator, or nothing when it is empty. Display only, so no events.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<HotkeyhintEvent> {
        let names: Vec<&str> = self.p.accel.split('+').map(str::trim).filter(|s| !s.is_empty()).collect();
        let Some((key_name, modifiers)) = names.split_last() else {
            return vec![];
        };
        let platform = resolve_platform(&self.p.platform);
        let md = self.p.size == "md";
        // The web's sizes: sm 0.857rem glyphs and text on a 0.857rem line; md 0.929rem on a 1.286rem line.
        let (font_px, line) = if md { (rem(0.929), rem(1.286)) } else { (rem(0.857), rem(0.857)) };
        // inherit takes the parent's text colour (content_1 is the body text); muted is content_3.
        let color = if self.p.tone == "muted" { look.t.content_3 } else { look.t.content_1 };

        // Modifiers, then the key. A text modifier gets a small trailing gap so it reads as a word.
        let mut tokens: Vec<(Token, f32)> = modifiers
            .iter()
            .map(|m| {
                let t = modifier(m, platform);
                let gap = if t.glyph.is_none() { rem(0.214) } else { 0.0 };
                (t, gap)
            })
            .collect();
        tokens.push((key_token(key_name), 0.0));
        let spoken = tokens.iter().map(|(t, _)| t.label.as_str()).collect::<Vec<_>>().join(" ");

        let mut laid = Vec::with_capacity(tokens.len());
        let mut width = 0.0;
        for (token, gap) in tokens {
            let (text, font) = match token.glyph {
                Some(name) => (
                    icon(name, font_px).text().to_owned(),
                    FontId::new(font_px, FontFamily::Name("Icons".into())),
                ),
                None => (token.text, FontId::new(font_px, weight(300))),
            };
            let galley = ui.painter().layout_no_wrap(text, font, color);
            width += galley.size().x + gap;
            laid.push((galley, gap));
        }

        let (rect, response) = ui.allocate_exact_size(egui::vec2(width, line), Sense::hover());
        response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, spoken.clone()));
        part(ui, &response, parts::HINT);

        let painter = ui.painter();
        let mut x = rect.left();
        for (galley, gap) in laid {
            let (w, h) = (galley.size().x, galley.size().y);
            painter.galley(egui::pos2(x, rect.center().y - h / 2.0), galley, color);
            x += w + gap;
        }
        vec![]
    }
}

crate::mounted!(Hotkeyhint, HotkeyhintProps);
