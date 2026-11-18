//! The menu-bar (macOS) or notification-area (Windows) icon: the reference's tray agent, with Open Taskscape
//! and Quit Taskscape. While it is there, closing the main window hides it and the app keeps running for the
//! Global hotkeys; Quit ends it. Linux has none (tray-icon needs GTK there), so closing the window quits.

/// What the tray's menu asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayAction {
    OpenMain,
    Quit,
}

/// The icon: a ring with a tick, drawn at `size` px as alpha over one colour: black for macOS's template image
/// (the menu bar tints it for light and dark), white for Windows's notification area (dark by default).
pub fn icon_rgba(size: u32, ink: u8) -> Vec<u8> {
    let s = size as f32;
    let (c, r, w) = (s / 2.0, s * 0.42, s * 0.085);
    // The tick, as two segments.
    let tick = [((s * 0.30, s * 0.52), (s * 0.44, s * 0.66)), ((s * 0.44, s * 0.66), (s * 0.70, s * 0.36))];
    let segment = |(px, py): (f32, f32), ((ax, ay), (bx, by)): ((f32, f32), (f32, f32))| {
        let (dx, dy) = (bx - ax, by - ay);
        let t = (((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        ((px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2)).sqrt()
    };
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let p = (x as f32 + 0.5, y as f32 + 0.5);
            let ring = ((p.0 - c).hypot(p.1 - c) - r).abs();
            let d = tick.iter().map(|&seg| segment(p, seg)).fold(ring, f32::min);
            // A pixel's coverage of the stroke, soft at its edge.
            let a = (w + 0.5 - d).clamp(0.0, 1.0);
            out.extend_from_slice(&[ink, ink, ink, (a * 255.0) as u8]);
        }
    }
    out
}

#[cfg(all(feature = "window", any(target_os = "macos", target_os = "windows")))]
pub use os::Tray;

#[cfg(all(feature = "window", any(target_os = "macos", target_os = "windows")))]
mod os {
    use super::{icon_rgba, TrayAction};
    use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
    use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

    pub struct Tray {
        _icon: TrayIcon,
        open: MenuId,
        quit: MenuId,
    }

    impl Tray {
        /// None when the OS won't show it (logged).
        pub fn new() -> Option<Self> {
            let open = MenuItem::new("Open Taskscape", true, None);
            let quit = MenuItem::new("Quit Taskscape", true, None);
            let menu = Menu::new();
            if menu.append_items(&[&open, &PredefinedMenuItem::separator(), &quit]).is_err() {
                return None;
            }
            let builder = TrayIconBuilder::new().with_menu(Box::new(menu)).with_tooltip("Taskscape");
            #[cfg(target_os = "macos")]
            let builder = builder.with_icon_templated(Icon::from_rgba(icon_rgba(44, 0), 44, 44).ok()?);
            #[cfg(not(target_os = "macos"))]
            let builder = builder.with_icon(Icon::from_rgba(icon_rgba(32, 255), 32, 32).ok()?);
            let built = builder.build();
            match built {
                Ok(icon) => Some(Self { _icon: icon, open: open.id().clone(), quit: quit.id().clone() }),
                Err(e) => {
                    eprintln!("taskscape: the tray icon is unavailable: {e}");
                    None
                }
            }
        }

        /// The tray menu's choices since the last frame.
        pub fn poll(&self) -> Vec<TrayAction> {
            MenuEvent::receiver()
                .try_iter()
                .filter_map(|e| {
                    if e.id == self.open {
                        Some(TrayAction::OpenMain)
                    } else if e.id == self.quit {
                        Some(TrayAction::Quit)
                    } else {
                        None
                    }
                })
                .collect()
        }
    }
}

/// Linux (and the screen tests): no tray, so the main window's close quits the app.
#[cfg(not(all(feature = "window", any(target_os = "macos", target_os = "windows"))))]
pub struct Tray;

#[cfg(not(all(feature = "window", any(target_os = "macos", target_os = "windows"))))]
impl Tray {
    pub fn new() -> Option<Self> {
        None
    }

    pub fn poll(&self) -> Vec<TrayAction> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_icon_is_a_template_with_a_ring_and_a_tick() {
        let size = 44;
        let px = icon_rgba(size, 0);
        assert_eq!(px.len(), (size * size * 4) as usize);
        let alpha = |x: u32, y: u32| px[((y * size + x) * 4 + 3) as usize];
        // One colour, shaped by alpha.
        assert!(px.chunks_exact(4).all(|p| p[0] == 0 && p[1] == 0 && p[2] == 0));
        assert!(icon_rgba(32, 255).chunks_exact(4).all(|p| p[0] == 255));
        // The ring at its radius (left edge), clear inside it, clear in the corners.
        assert!(alpha(22 - 18, 22) > 200);
        assert_eq!(alpha(22 - 8, 22 - 8), 0);
        assert_eq!(alpha(0, 0), 0);
        // The tick's corner.
        assert!(alpha((44.0 * 0.44) as u32, (44.0 * 0.66) as u32) > 200);
    }
}
