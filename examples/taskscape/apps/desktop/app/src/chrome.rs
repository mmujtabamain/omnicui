//! The app's own window chrome, as the reference has it: each window's bar is its title bar, with the window
//! buttons inside it. On macOS the OS draws its own lights over the app's bar (a transparent title bar; the
//! reference draws its own, but Mujtaba chose the OS's: the reference is outdated there). On Windows and Linux
//! the windows have no OS frame and the app draws minimise, maximise and close. Settings, Trash and the sheets cannot be minimised
//! or zoomed. Screen tests draw no chrome (`App::native_window` is off there).
use egui::{pos2, vec2, Align2, CornerRadius, FontFamily, FontId, Id, Rect, Sense, Ui, ViewportBuilder, ViewportCommand};
use taskscape_ui::{icon, Look};

pub const MAC: bool = cfg!(target_os = "macos");

/// The height of the main and panel title bars in egui points.
pub const BAR_HEIGHT: f32 = 46.0;

pub(crate) const PROJECT_INSET: f32 = 12.0;
pub(crate) const SETTINGS_TITLE_INSET: f32 = taskscape_ui::rem(1.286);
pub(crate) const TRASH_TITLE_INSET: f32 = taskscape_ui::rem(1.0);

pub const MAIN_TITLE: &str = "Taskscape";
pub(crate) const SETTINGS_TITLE: &str = "Settings";
pub(crate) const TRASH_TITLE: &str = "Trash";

/// The width the OS's window buttons take at the left of an overlay title bar (macOS), with their margin.
pub const LIGHTS_W: f32 = 78.0;

/// The width of the app's own window buttons at the right of a bar (Windows, Linux): one per button.
pub const BUTTON_W: f32 = 40.0;

/// The main window: the app's bar is the title bar.
pub fn main_window(vb: ViewportBuilder) -> ViewportBuilder {
    if MAC {
        vb.with_fullsize_content_view(true).with_titlebar_shown(false).with_title_shown(false)
    } else {
        vb.with_decorations(false)
    }
}

/// Settings and Trash: their header is the title bar; no minimise, no zoom.
pub fn panel(vb: ViewportBuilder) -> ViewportBuilder {
    let vb = vb.with_minimize_button(false).with_maximize_button(false);
    if MAC {
        vb.with_fullsize_content_view(true).with_titlebar_shown(false).with_title_shown(false)
    } else {
        vb.with_decorations(false)
    }
}

/// A confirmation or prompt: a card of its own with no frame around it, as the reference's sheet.
pub fn sheet(vb: ViewportBuilder) -> ViewportBuilder {
    vb.with_decorations(false).with_transparent(true).with_minimize_button(false).with_maximize_button(false).with_resizable(false)
}

/// What a builder asks the OS for, as a test reads it (see taskscape_ui::WindowKind).
pub fn kind(vb: &ViewportBuilder) -> taskscape_ui::WindowKind {
    let app_bar = vb.decorations == Some(false) || (vb.fullsize_content_view == Some(true) && vb.titlebar_shown == Some(false));
    taskscape_ui::WindowKind {
        title_bar: if app_bar { "app" } else { "os" },
        minimizable: vb.minimize_button.unwrap_or(true),
        maximizable: vb.maximize_button.unwrap_or(true),
    }
}

/// Space to keep free at the left of a window's bar for the OS's buttons (macOS overlay), else none.
/// None in full screen, where macOS hides the lights (it shows them only with the menu bar).
pub fn lead(ui: &Ui, native: bool) -> f32 {
    let full_screen = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
    if native && MAC && !full_screen {
        LIGHTS_W
    } else {
        0.0
    }
}

/// Space to keep free at the right of a window's bar for the app's own buttons (Windows, Linux).
pub fn trail(native: bool, buttons: usize) -> f32 {
    if native && !MAC {
        BUTTON_W * buttons as f32
    } else {
        0.0
    }
}

fn no_drag_id(bar: Rect) -> Id {
    Id::unique(("chrome.noDrag", bar.min.x as i32, bar.min.y as i32))
}

/// A control in a bar: a press on it never moves the window, even where egui would hand the drag to the bar.
pub fn no_drag(ui: &Ui, bar: Rect, control: Rect) {
    ui.ctx().data_mut(|d| d.get_temp_mut_or_default::<Vec<Rect>>(no_drag_id(bar)).push(control));
}

/// The bar as a drag region: dragging its empty space moves the window, a double click zooms the main
/// window. Registered before the bar's controls, which sit above it and keep their presses; a press that
/// began on a control the bar named last frame (`no_drag`) never moves the window (a dragged tab moved
/// the window on macOS, #12480).
pub fn drag_region(ui: &mut Ui, bar: Rect, zoomable: bool) {
    let controls = ui.ctx().data_mut(|d| std::mem::take(d.get_temp_mut_or_default::<Vec<Rect>>(no_drag_id(bar))));
    let r = ui.interact(bar, Id::unique(("chrome.drag", bar.min.x as i32, bar.min.y as i32)), Sense::click_and_drag());
    let origin = ui.input(|i| i.pointer.press_origin());
    let on_control = origin.is_some_and(|p| controls.iter().any(|c| c.contains(p)));
    if r.drag_started() && !on_control {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }
    if zoomable && r.double_clicked() {
        let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!max));
    }
}

/// The app's own window buttons at the right end of a bar (Windows, Linux): minimise and maximise for
/// the main window, close for every window. Returns whether close was pressed (the caller closes its window).
pub fn buttons(ui: &mut Ui, look: Look, bar: Rect, minimise: bool) -> bool {
    let t = look.t;
    let names: &[(&str, &str)] = if minimise { &[("remove", "Minimise"), ("crop_square", "Maximise"), ("close", "Close")] } else { &[("close", "Close")] };
    let mut x = bar.right() - BUTTON_W * names.len() as f32;
    let mut closed = false;
    for (glyph, name) in names {
        let r = Rect::from_min_size(pos2(x, bar.top()), vec2(BUTTON_W, bar.height()));
        x += BUTTON_W;
        let resp = ui.interact(r, Id::unique(("chrome.button", *name, bar.min.y as i32)), Sense::click());
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, *name));
        if resp.hovered() {
            let fill = if *name == "Close" { t.danger_500 } else { t.wash_2 };
            ui.painter().rect_filled(r, CornerRadius::ZERO, fill);
        }
        let ink = if resp.hovered() && *name == "Close" { t.on_accent } else { t.content_2 };
        ui.painter().text(r.center(), Align2::CENTER_CENTER, icon(glyph, 16.0).text(), FontId::new(16.0, FontFamily::Name("Icons".into())), ink);
        if resp.clicked() {
            match *name {
                "Minimise" => ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true)),
                "Maximise" => {
                    let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                    ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!max));
                }
                _ => closed = true,
            }
        }
    }
    closed
}

/// macOS: the main window keeps a (transparent) title bar for its rounded frame, and AppKit would move the
/// window from that strip on its own, under the app's tabs and buttons (#12480). It is made not movable by
/// the OS; the app's drag region moves it instead (performWindowDrag works on a window that isn't movable).
#[cfg(all(target_os = "macos", feature = "window"))]
pub fn not_movable_by_os(handle: &impl raw_window_handle::HasWindowHandle) {
    use raw_window_handle::RawWindowHandle;
    let Ok(h) = handle.window_handle() else { return };
    if let RawWindowHandle::AppKit(h) = h.as_raw() {
        // SAFETY: the handle's view is the window's live NSView, on the main thread.
        let view: &objc2_app_kit::NSView = unsafe { h.ns_view.cast().as_ref() };
        if let Some(window) = view.window() {
            window.setMovable(false);
        }
    }
}

/// Center the native traffic lights in the space before the first title bar item.
/// AppKit can relayout the buttons during resize, so check their positions each frame.
#[cfg(all(target_os = "macos", feature = "window"))]
pub fn align_window_controls(zoom: f32) {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSWindowButton, NSWindowStyleMask};

    let Some(main_thread) = MainThreadMarker::new() else { return };
    // The immediate Settings and Trash viewports don't expose a window handle
    // to the caller. Use the app's own windows and select its overlay title bars.
    for window in NSApplication::sharedApplication(main_thread).windows().iter() {
        let style = window.styleMask();
        if !window.isVisible() || !window.titlebarAppearsTransparent()
            || !style.contains(NSWindowStyleMask::Titled | NSWindowStyleMask::FullSizeContentView)
            || style.contains(NSWindowStyleMask::FullScreen)
        {
            continue;
        }
        let inset = match window.title().to_string().as_str() {
            MAIN_TITLE => PROJECT_INSET,
            SETTINGS_TITLE => SETTINGS_TITLE_INSET,
            TRASH_TITLE => TRASH_TITLE_INSET,
            _ => continue,
        };
        let Some(content) = window.contentView() else { continue };
        let content = content.convertRect_toView(content.bounds(), None);
        let slot_width = f64::from((LIGHTS_W + inset) * zoom);
        let center_y = content.origin.y + content.size.height - f64::from(BAR_HEIGHT * zoom) / 2.0;
        let buttons = [NSWindowButton::CloseButton, NSWindowButton::MiniaturizeButton, NSWindowButton::ZoomButton]
            .map(|kind| window.standardWindowButton(kind).filter(|button| !button.isHiddenOrHasHiddenAncestor()));
        let (left, right) = buttons.iter().flatten().fold((f64::INFINITY, f64::NEG_INFINITY), |(left, right), button| {
            let rect = button.convertRect_toView(button.bounds(), None);
            (left.min(rect.origin.x), right.max(rect.origin.x + rect.size.width))
        });
        let group_left = content.origin.x + lights_left(slot_width, right - left);
        let offset_x = group_left - left;
        for button in buttons.into_iter().flatten() {
            // SAFETY: used immediately, on the main thread, while the window owns the button.
            let Some(parent) = (unsafe { button.superview() }) else { continue };
            let frame = button.frame();
            let rect = button.convertRect_toView(button.bounds(), None);
            let mut center = rect.origin;
            center.x += rect.size.width / 2.0 + offset_x;
            center.y = center_y;
            let mut origin = parent.convertPoint_fromView(center, None);
            origin.x -= frame.size.width / 2.0;
            origin.y -= frame.size.height / 2.0;
            if (frame.origin.x - origin.x).abs() > 0.01 || (frame.origin.y - origin.y).abs() > 0.01 {
                button.setFrameOrigin(origin);
            }
        }
    }
}

/// Where the lights' group starts in the slot before a bar's first item: centred, keeping the buttons'
/// native spacing; at very small zooms, at least 4 points in, clear of the window's rounded corner.
pub fn lights_left(slot_width: f64, group_width: f64) -> f64 {
    ((slot_width - group_width) / 2.0).max(4.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lights_sit_centred_in_their_slot() {
        // The main window at 1×: the lights' slot is LIGHTS_W + the pill's inset; AppKit's group is 52 points.
        let slot = f64::from(LIGHTS_W + PROJECT_INSET);
        assert_eq!(lights_left(slot, 52.0), (slot - 52.0) / 2.0);
        // Settings: its title's inset.
        let slot = f64::from(LIGHTS_W + SETTINGS_TITLE_INSET);
        assert!((lights_left(slot, 52.0) * 2.0 + 52.0 - slot).abs() < 1e-9);
        // A slot narrower than the group (a tiny zoom): never closer than 4 points to the corner.
        assert_eq!(lights_left(40.0, 52.0), 4.0);
    }
}
