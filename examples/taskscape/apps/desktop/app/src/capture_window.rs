//! The capture bar in a window of its own: the reference's mini bar, summoned from anywhere by the Global
//! hotkeys (Capture bar, Screenshot capture). Always on top, frameless, exactly as tall as the card, opened
//! just down-right of the cursor (macOS) or over the main window; a click away dismisses it and keeps the draft.
use crate::screens::capturebar::CaptureBar;
use crate::App;
use egui::{pos2, vec2, Context, Pos2, Rect, Vec2, ViewportBuilder, ViewportCommand, ViewportId};
use taskscape_ui::omnicui::capturebar::CapturebarEvent;
use taskscape_ui::Look;

/// The bar's width (the reference's window).
pub const BAR_W: f32 = 480.0;
/// The tallest the card grows (its notes open): the vertical flip is decided against it, so a bar opened near
/// the bottom of the screen has room to grow (the reference's MAX_BAR_HEIGHT).
pub const MAX_BAR_H: f32 = 260.0;
/// A blur this soon after opening is the window settling, not a click away (the reference's REVEAL_GRACE).
const GRACE: f64 = 0.5;
/// How long the bar is off screen before the grab, so the shot doesn't catch it.
const HIDE_FOR_GRAB: f64 = 0.25;
const TITLE: &str = "Taskscape Capture";

fn id() -> ViewportId {
    ViewportId::from_hash_of("capture")
}

pub struct CaptureWindow {
    bar: CaptureBar,
    open: bool,
    /// Where the window opens, in screen points (top-left), chosen when it is summoned.
    at: Pos2,
    shown_at: f64,
    /// A grab in progress: when the bar went off screen for it.
    hidden_for_grab: Option<f64>,
    /// The grab, running on a thread of its own: an interactive region capture waits on the user, and the
    /// window must keep drawing meanwhile (the reference's background capture).
    grab: Option<std::sync::mpsc::Receiver<Result<Option<String>, ()>>>,
}

impl Default for CaptureWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureWindow {
    pub fn new() -> Self {
        let mut bar = CaptureBar::new();
        bar.use_window_capture();
        Self { bar, open: false, at: Pos2::ZERO, shown_at: 0.0, hidden_for_grab: None, grab: None }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Capture bar: shows the bar, or hides it when it is up.
    pub fn toggle(&mut self, app: &mut App, ctx: &Context) {
        if self.open {
            self.close(app);
        } else {
            self.summon(app, ctx);
        }
    }

    /// Screenshot capture: a shot of the screen without the bar, attached to the draft; the bar is shown.
    pub fn screenshot(&mut self, app: &mut App, ctx: &Context) {
        if !self.open {
            self.summon(app, ctx);
        }
        let _ = self.bar.request_shot();
    }

    fn summon(&mut self, app: &mut App, ctx: &Context) {
        self.at = placement(ctx);
        self.bar.reveal();
        self.open = true;
        self.shown_at = ctx.input(|i| i.time);
        app.capture_open = true;
        ctx.send_viewport_cmd_to(id(), ViewportCommand::Focus);
        ctx.request_repaint();
    }

    fn close(&mut self, app: &mut App) {
        let _ = self.bar.dismiss(app);
        self.open = false;
        app.capture_open = false;
    }

    /// One frame of the main window's loop: the bar's window, and a grab in progress.
    pub fn show(&mut self, app: &mut App, ctx: &Context, look: Look) {
        let now = ctx.input(|i| i.time);
        if self.bar.take_shot_request() && self.hidden_for_grab.is_none() {
            self.hidden_for_grab = Some(now);
            ctx.send_viewport_cmd_to(id(), ViewportCommand::Visible(false));
        }
        if let Some(since) = self.hidden_for_grab {
            if self.grab.is_none() && now - since >= HIDE_FOR_GRAB {
                let (tx, rx) = std::sync::mpsc::channel();
                let wake = ctx.clone();
                std::thread::spawn(move || {
                    let result = match taskscape_storage::screenshot::capture() {
                        Ok(path) => Ok(path.map(|p| p.to_string_lossy().into_owned())),
                        Err(e) => {
                            eprintln!("taskscape: the screenshot failed: {e:#}");
                            Err(())
                        }
                    };
                    let _ = tx.send(result);
                    wake.request_repaint();
                });
                self.grab = Some(rx);
            }
            let done = match self.grab.as_ref().map(|rx| rx.try_recv()) {
                Some(Ok(result)) => Some(result),
                Some(Err(std::sync::mpsc::TryRecvError::Disconnected)) => Some(Err(())),
                _ => None,
            };
            if let Some(result) = done {
                self.grab = None;
                self.hidden_for_grab = None;
                self.bar.finish_shot(result);
                ctx.send_viewport_cmd_to(id(), ViewportCommand::Visible(true));
                ctx.send_viewport_cmd_to(id(), ViewportCommand::Focus);
                self.shown_at = now;
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            }
        }
        if !self.open {
            return;
        }
        let height = self.bar.card_height().max(56.0);
        let builder = ViewportBuilder::default()
            .with_title(TITLE)
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            .with_minimize_button(false)
            .with_maximize_button(false)
            .with_always_on_top()
            .with_taskbar(false)
            .with_inner_size([BAR_W, height])
            .with_position(self.at)
            .with_active(true);
        let mut main_requested = false;
        let mut closed = false;
        ctx.show_viewport_immediate(id(), builder, |ctx, _| {
            egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| {
                for e in self.bar.show(app, ui, look) {
                    main_requested |= matches!(e, CapturebarEvent::MainRequested { .. });
                }
            });
            // The window is exactly as tall as the card: any part it didn't paint would take clicks meant for
            // the app underneath.
            let want = self.bar.card_height();
            if want > 0.0 && ctx.input(|i| i.viewport().inner_rect).is_some_and(|r| (r.height() - want).abs() > 0.5) {
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(BAR_W, want)));
            }
            let focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
            let settled = ctx.input(|i| i.time) - self.shown_at > GRACE;
            if (!focused && settled && self.hidden_for_grab.is_none()) || ctx.input(|i| i.viewport().close_requested()) {
                closed = true;
            }
        });
        if closed || !self.bar.is_visible() {
            self.close(app);
        }
        if main_requested {
            ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Visible(true));
            ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Focus);
        }
    }
}

/// Where the bar opens. On macOS just down-right of the cursor, on the screen it is on; elsewhere (egui can't
/// read the cursor outside its windows) centred over the main window's upper part.
fn placement(ctx: &Context) -> Pos2 {
    #[cfg(all(target_os = "macos", feature = "window"))]
    if let Some((cursor, work)) = macos::cursor_and_work_area() {
        return anchor(cursor, vec2(BAR_W, 56.0), MAX_BAR_H, work);
    }
    let main = ctx.input(|i| i.viewport().outer_rect).unwrap_or(Rect::from_min_size(Pos2::ZERO, vec2(1180.0, 760.0)));
    pos2(main.center().x - BAR_W / 2.0, main.top() + main.height() / 4.0)
}

/// The bar's top-left: just down-right of the cursor, flipped to the cursor's other side when it would cross the
/// work area's right or bottom edge (the bottom judged against the tallest the card grows), then kept inside the
/// work area (the reference's cursor_anchor).
pub fn anchor(cursor: Pos2, size: Vec2, max_h: f32, work: Rect) -> Pos2 {
    const GAP: f32 = 8.0;
    let mut x = cursor.x + GAP;
    if x + size.x > work.right() {
        x = cursor.x - GAP - size.x;
    }
    let mut y = cursor.y + GAP;
    if y + max_h > work.bottom() {
        y = cursor.y - GAP - size.y;
    }
    pos2(x.clamp(work.left(), (work.right() - size.x).max(work.left())), y.clamp(work.top(), (work.bottom() - size.y).max(work.top())))
}

#[cfg(all(target_os = "macos", feature = "window"))]
mod macos {
    use egui::{pos2, Pos2, Rect};
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSEvent, NSScreen};

    /// The cursor and the work area (menu bar and Dock excluded) of the screen it is on, in top-left points.
    pub fn cursor_and_work_area() -> Option<(Pos2, Rect)> {
        let mtm = MainThreadMarker::new()?;
        let p = NSEvent::mouseLocation();
        let screens = NSScreen::screens(mtm);
        // AppKit's y runs up from the bottom of the first screen; egui's runs down from its top.
        let top = screens.iter().next()?.frame().size.height;
        let screen = screens.iter().find(|s| {
            let f = s.frame();
            p.x >= f.origin.x && p.x < f.origin.x + f.size.width && p.y >= f.origin.y && p.y < f.origin.y + f.size.height
        })?;
        let v = screen.visibleFrame();
        let work = Rect::from_min_size(
            pos2(v.origin.x as f32, (top - v.origin.y - v.size.height) as f32),
            egui::vec2(v.size.width as f32, v.size.height as f32),
        );
        Some((pos2(p.x as f32, (top - p.y) as f32), work))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORK: Rect = Rect { min: Pos2 { x: 0.0, y: 25.0 }, max: Pos2 { x: 1440.0, y: 900.0 } };
    const SIZE: Vec2 = Vec2 { x: BAR_W, y: 56.0 };

    #[test]
    fn down_right_of_the_cursor() {
        assert_eq!(anchor(pos2(300.0, 200.0), SIZE, MAX_BAR_H, WORK), pos2(308.0, 208.0));
    }

    #[test]
    fn flipped_left_near_the_right_edge() {
        assert_eq!(anchor(pos2(1200.0, 200.0), SIZE, MAX_BAR_H, WORK), pos2(1200.0 - 8.0 - BAR_W, 208.0));
    }

    #[test]
    fn flipped_up_when_the_grown_card_would_cross_the_bottom() {
        // Room for the 56 px bar below, not for the notes it may grow into.
        assert_eq!(anchor(pos2(300.0, 700.0), SIZE, MAX_BAR_H, WORK), pos2(308.0, 700.0 - 8.0 - 56.0));
    }

    #[test]
    fn kept_inside_a_small_work_area() {
        // A 400 px wide screen: flipped left would leave it, so it's held at the left edge.
        let small = Rect::from_min_size(pos2(0.0, 25.0), vec2(400.0, 300.0));
        assert_eq!(anchor(pos2(5.0, 30.0), SIZE, MAX_BAR_H, small), pos2(0.0, 38.0));
        // The bottom-right corner: flipped left and up, both inside.
        assert_eq!(anchor(pos2(1430.0, 890.0), SIZE, MAX_BAR_H, WORK), pos2(1430.0 - 8.0 - BAR_W, 890.0 - 8.0 - 56.0));
    }
}
