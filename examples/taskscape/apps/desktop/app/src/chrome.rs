//! The app's own window chrome, as the reference has it: each window's bar is its title bar, with the window
//! buttons inside it. On macOS the OS draws its buttons over the app's bar (an overlay title bar); on Windows
//! and Linux the windows have no OS frame and the app draws the buttons. Settings, Trash and the sheets cannot
//! be minimised or zoomed. Screen tests draw no chrome (`App::native_window` is off there).
use egui::{pos2, vec2, Align2, CornerRadius, FontFamily, FontId, Id, Rect, Sense, Ui, ViewportBuilder, ViewportCommand};
use taskscape_ui::{icon, Look};

pub const MAC: bool = cfg!(target_os = "macos");

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

/// Space to keep free at the left of a window's bar for the OS's buttons (macOS overlay), else none.
pub fn lead(native: bool) -> f32 {
    if native && MAC {
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

/// The bar as a drag region: dragging its empty space moves the window, a double click zooms the main
/// window. Registered before the bar's controls, which sit above it and keep their presses.
pub fn drag_region(ui: &mut Ui, bar: Rect, zoomable: bool) {
    let r = ui.interact(bar, Id::new(("chrome.drag", bar.min.x as i32, bar.min.y as i32)), Sense::click_and_drag());
    if r.drag_started() {
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
        let resp = ui.interact(r, Id::new(("chrome.button", *name, bar.min.y as i32)), Sense::click());
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
