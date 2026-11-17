//! The macOS menu bar, as the reference's (Tauri's standard menu with a List menu added): the App menu,
//! View (full screen), Window (Minimize ⌘M, Zoom, Close ⌘W) and List (import and export). There is no Edit
//! menu: its ⌘C/⌘V would go to the menu, and egui's text fields take those keys themselves.

/// What a menu item asks the app to do.
pub enum MenuAction {
    ImportList,
    ExportList,
}

#[cfg(all(target_os = "macos", feature = "window"))]
pub struct NativeMenu {
    _menu: muda::Menu,
    import: muda::MenuId,
    export: muda::MenuId,
}

#[cfg(all(target_os = "macos", feature = "window"))]
impl NativeMenu {
    /// Builds the menu bar and gives it to the app (on the main thread, once the app has started).
    pub fn install() -> Option<Self> {
        use muda::{Menu, MenuItem, PredefinedMenuItem as P, Submenu};
        let import = MenuItem::new("Import List from JSON…", true, None);
        let export = MenuItem::new("Export List to JSON…", true, None);
        let app = Submenu::with_items(
            "Taskscape",
            true,
            &[&P::about(None, None), &P::separator(), &P::services(None), &P::separator(), &P::hide(None), &P::hide_others(None), &P::show_all(None), &P::separator(), &P::quit(None)],
        )
        .ok()?;
        let view = Submenu::with_items("View", true, &[&P::fullscreen(None)]).ok()?;
        let window = Submenu::with_items("Window", true, &[&P::minimize(None), &P::maximize(None), &P::separator(), &P::close_window(None)]).ok()?;
        let list = Submenu::with_items("List", true, &[&import, &export]).ok()?;
        let menu = Menu::with_items(&[&app, &view, &window, &list]).ok()?;
        menu.init_for_nsapp();
        window.set_as_windows_menu_for_nsapp();
        Some(Self { _menu: menu, import: import.id().clone(), export: export.id().clone() })
    }

    /// The menu item chosen since the last frame, if any.
    pub fn poll(&self) -> Option<MenuAction> {
        let event = muda::MenuEvent::receiver().try_recv().ok()?;
        if event.id == self.import {
            Some(MenuAction::ImportList)
        } else if event.id == self.export {
            Some(MenuAction::ExportList)
        } else {
            None
        }
    }
}

/// Elsewhere there is no OS menu bar: the app's bars carry the window buttons (see chrome).
#[cfg(not(all(target_os = "macos", feature = "window")))]
pub struct NativeMenu;

#[cfg(not(all(target_os = "macos", feature = "window")))]
impl NativeMenu {
    pub fn install() -> Option<Self> {
        None
    }
    pub fn poll(&self) -> Option<MenuAction> {
        None
    }
}
