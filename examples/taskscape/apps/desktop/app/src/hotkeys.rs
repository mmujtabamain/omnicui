//! The OS-wide hotkeys (Settings › Shortcuts › Global): Capture bar and Screenshot capture, registered with
//! the OS from their saved bindings and registered again when one changes.

/// What a global hotkey does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlobalAction {
    /// Shows the capture bar, or hides it when it is up.
    ToggleCaptureBar,
    /// Takes a screenshot and shows the capture bar with it attached.
    ScreenshotCapture,
}

/// The Global commands: the command id their binding is saved under, its default, and what it does.
pub const GLOBAL: [(&str, &str, GlobalAction); 2] = [
    ("toggle_capture_bar", "Mod+Enter", GlobalAction::ToggleCaptureBar),
    ("screenshot_capture", "Mod+Shift+Enter", GlobalAction::ScreenshotCapture),
];

/// A saved binding as the OS registers it: its modifiers and its key as a keyboard code name ("KeyN",
/// "Enter", "Digit1").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OsAccel {
    pub command: bool,
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub code: String,
}

/// A saved binding ("Mod+Shift+Enter") as the OS registers it, on macOS (`mac`, where Mod is Command) or
/// elsewhere (Mod is Control). None for a binding with no modifier or a key the OS can't take: a bare key
/// would swallow typing in every app.
pub fn os_accel(stored: &str, mac: bool) -> Option<OsAccel> {
    let cut = if stored.ends_with('+') { stored.len() - 1 } else { stored.rfind('+').map_or(0, |i| i + 1) };
    let (head, key) = stored.split_at(cut);
    let mut a = OsAccel { command: false, control: false, alt: false, shift: false, code: code_of(key)? };
    for m in head.split('+').filter(|m| !m.is_empty()) {
        match m {
            "Mod" if mac => a.command = true,
            "Mod" | "Ctrl" | "Control" => a.control = true,
            "Meta" | "Command" | "Cmd" => a.command = true,
            "Alt" | "Option" => a.alt = true,
            "Shift" => a.shift = true,
            _ => return None,
        }
    }
    (a.command || a.control || a.alt).then_some(a)
}

/// A key as a keyboard code name (the W3C UI Events names the OS hotkey crate reads).
fn code_of(key: &str) -> Option<String> {
    let named = match key {
        "Enter" | "Return" => "Enter",
        "Space" | " " => "Space",
        "Backspace" => "Backspace",
        "Delete" => "Delete",
        "Tab" => "Tab",
        "Escape" | "Esc" => "Escape",
        "ArrowLeft" => "ArrowLeft",
        "ArrowRight" => "ArrowRight",
        "ArrowUp" => "ArrowUp",
        "ArrowDown" => "ArrowDown",
        "," | "Comma" => "Comma",
        "." | "Period" => "Period",
        "/" | "Slash" => "Slash",
        "\\" | "Backslash" => "Backslash",
        ";" | "Semicolon" => "Semicolon",
        "'" | "Quote" => "Quote",
        "[" | "BracketLeft" => "BracketLeft",
        "]" | "BracketRight" => "BracketRight",
        "-" | "Minus" => "Minus",
        "=" | "Equal" => "Equal",
        "`" | "Backquote" => "Backquote",
        _ => "",
    };
    if !named.is_empty() {
        return Some(named.to_owned());
    }
    let mut chars = key.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_alphabetic() => Some(format!("Key{}", c.to_ascii_uppercase())),
        (Some(c), None) if c.is_ascii_digit() => Some(format!("Digit{c}")),
        _ if key.len() <= 3 && key.starts_with('F') && key[1..].parse::<u8>().is_ok_and(|n| (1..=24).contains(&n)) => Some(key.to_owned()),
        _ => None,
    }
}

/// The saved binding of a Global command, or its default.
pub fn binding(app: &crate::App, command: &str, default: &str) -> String {
    app.data.get_setting(&crate::screens::settingsshortcuts::setting_key(command)).unwrap_or_else(|| default.to_owned())
}

#[cfg(feature = "window")]
pub use registry::GlobalHotkeys;

#[cfg(feature = "window")]
mod registry {
    use super::{binding, os_accel, GlobalAction, GLOBAL};
    use global_hotkey::hotkey::{Code, HotKey, Modifiers};
    use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
    use std::str::FromStr;
    use std::sync::mpsc;

    /// The registered Global hotkeys. A press wakes the app (the OS calls in from outside its event loop).
    pub struct GlobalHotkeys {
        manager: GlobalHotKeyManager,
        /// Per Global command: the binding registered, and its hotkey (None when it couldn't be).
        bound: Vec<(String, Option<HotKey>)>,
        presses: mpsc::Receiver<u32>,
    }

    impl GlobalHotkeys {
        /// None when the OS has no global hotkeys to offer (a Linux session without X11, for one).
        pub fn new(ctx: &egui::Context) -> Option<Self> {
            let manager = match GlobalHotKeyManager::new() {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("taskscape: global hotkeys are unavailable: {e}");
                    return None;
                }
            };
            let (tx, presses) = mpsc::channel();
            let ctx = ctx.clone();
            GlobalHotKeyEvent::set_event_handler(Some(move |e: GlobalHotKeyEvent| {
                if e.state == HotKeyState::Pressed {
                    let _ = tx.send(e.id);
                    ctx.request_repaint();
                }
            }));
            Some(Self { manager, bound: vec![(String::new(), None); GLOBAL.len()], presses })
        }

        /// Registers each Global command's saved binding, again whenever Settings has changed it.
        pub fn sync(&mut self, app: &crate::App) {
            for (i, (command, default, _)) in GLOBAL.iter().enumerate() {
                let accel = binding(app, command, default);
                if self.bound[i].0 == accel {
                    continue;
                }
                if let Some(old) = self.bound[i].1.take() {
                    let _ = self.manager.unregister(old);
                }
                let hotkey = os_accel(&accel, cfg!(target_os = "macos")).and_then(|a| {
                    let mut mods = Modifiers::empty();
                    mods.set(Modifiers::META, a.command);
                    mods.set(Modifiers::CONTROL, a.control);
                    mods.set(Modifiers::ALT, a.alt);
                    mods.set(Modifiers::SHIFT, a.shift);
                    Code::from_str(&a.code).ok().map(|code| HotKey::new(Some(mods), code))
                });
                match hotkey.map(|h| (h, self.manager.register(h))) {
                    Some((h, Ok(()))) => self.bound[i].1 = Some(h),
                    // Another app holds the combination: the command stays unbound, as the reference's does.
                    Some((_, Err(e))) => eprintln!("taskscape: {command} ({accel}) could not be registered: {e}"),
                    None => eprintln!("taskscape: {command} ({accel}) is not a combination the OS takes"),
                }
                self.bound[i].0 = accel;
            }
        }

        /// The Global commands pressed since the last frame.
        pub fn pressed(&self) -> Vec<GlobalAction> {
            let ids: Vec<u32> = self.presses.try_iter().collect();
            ids.into_iter()
                .filter_map(|id| self.bound.iter().position(|(_, h)| h.is_some_and(|h| h.id() == id)).map(|i| GLOBAL[i].2))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(command: bool, control: bool, alt: bool, shift: bool, code: &str) -> Option<OsAccel> {
        Some(OsAccel { command, control, alt, shift, code: code.to_owned() })
    }

    #[test]
    fn the_defaults_on_each_platform() {
        assert_eq!(os_accel("Mod+Enter", true), a(true, false, false, false, "Enter"));
        assert_eq!(os_accel("Mod+Enter", false), a(false, true, false, false, "Enter"));
        assert_eq!(os_accel("Mod+Shift+Enter", true), a(true, false, false, true, "Enter"));
        assert_eq!(os_accel("Mod+Shift+Enter", false), a(false, true, false, true, "Enter"));
    }

    #[test]
    fn rebound_keys() {
        assert_eq!(os_accel("Mod+N", true), a(true, false, false, false, "KeyN"));
        assert_eq!(os_accel("Mod+1", false), a(false, true, false, false, "Digit1"));
        assert_eq!(os_accel("Mod+Alt+ArrowLeft", true), a(true, false, true, false, "ArrowLeft"));
        assert_eq!(os_accel("Ctrl+Shift+,", true), a(false, true, false, true, "Comma"));
        assert_eq!(os_accel("Mod+\\", false), a(false, true, false, false, "Backslash"));
        assert_eq!(os_accel("Alt+F5", false), a(false, false, true, false, "F5"));
        assert_eq!(os_accel("Mod++", false), None, "a key the OS can't take");
    }

    #[test]
    fn a_bare_key_or_shift_alone_is_not_global() {
        assert_eq!(os_accel("Enter", true), None);
        assert_eq!(os_accel("Shift+N", false), None);
    }

    #[test]
    fn every_default_converts() {
        for (_, default, _) in GLOBAL {
            assert!(os_accel(default, true).is_some() && os_accel(default, false).is_some(), "{default}");
        }
    }
}
