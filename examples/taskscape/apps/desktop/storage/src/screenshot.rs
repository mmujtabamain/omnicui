use std::path::PathBuf;

use anyhow::Result;

/// The kind of grab [`capture`] performs, chosen by the user in Settings and
/// persisted as the `screenshot_mode` setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    /// Grab the whole screen instantly (`screencapture -x`).
    Fullscreen,
    /// Native interactive selection (`screencapture -i`): drag a region, press
    /// Space to pick a window, Esc to cancel.
    Region,
}

/// The settings key backing [`CaptureMode`].
pub const SCREENSHOT_MODE_KEY: &str = "screenshot_mode";

impl CaptureMode {
    /// Read the persisted `screenshot_mode` setting, defaulting to
    /// [`Fullscreen`](CaptureMode::Fullscreen) when unset or unrecognized.
    pub fn current() -> Self {
        match crate::settings::get(SCREENSHOT_MODE_KEY).as_deref() {
            Some("region") => CaptureMode::Region,
            _ => CaptureMode::Fullscreen,
        }
    }
}

/// Capture the screen per the user's [`CaptureMode`] and save it as a PNG under
/// `~/.taskscape/screenshots/`, returning the path to the saved file.
///
/// Returns `Ok(None)` when a region capture is cancelled (Esc / empty
/// selection) — the caller treats that as a silent no-op, not an error.
///
/// On macOS this shells out to the built-in `screencapture` tool. The first time
/// a given app triggers this, macOS will prompt the user to grant Screen
/// Recording permission to that app.
#[cfg(target_os = "macos")]
pub fn capture() -> Result<Option<PathBuf>> {
    use anyhow::bail;
    use std::process::Command;

    crate::paths::ensure_dirs()?;
    let path = crate::paths::screenshots_dir()
        .join(format!("screenshot-{}.png", crate::util::now_millis()));

    // -x: do not play the capture sound. -i (region only): interactive selection.
    let mut cmd = Command::new("screencapture");
    cmd.arg("-x");
    if CaptureMode::current() == CaptureMode::Region {
        cmd.arg("-i");
    }
    let status = cmd.arg(&path).status()?;

    if !status.success() {
        bail!("screencapture exited with status {:?}", status.code());
    }

    // Interactive capture exits successfully but writes no file when the user
    // cancels; report that as a no-op so it never surfaces as a broken shot.
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(path))
}

/// On Linux under X11 the root window is grabbed and written as a PNG, with no external tool; under Wayland,
/// where an app can't read the screen, the desktop's own tool takes it (grim, gnome-screenshot, spectacle). Both
/// take the whole screen.
#[cfg(target_os = "linux")]
pub fn capture() -> Result<Option<PathBuf>> {
    crate::paths::ensure_dirs()?;
    let path = crate::paths::screenshots_dir().join(format!("screenshot-{}.png", crate::util::now_millis()));
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return wayland_tool(&path);
    }
    x11_grab(&path)?;
    Ok(Some(path))
}

#[cfg(target_os = "linux")]
fn x11_grab(path: &std::path::Path) -> Result<()> {
    use anyhow::Context;
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{ConnectionExt, ImageFormat};

    let (conn, n) = x11rb::connect(None).context("no X11 display to capture")?;
    let screen = &conn.setup().roots[n];
    let (w, h) = (screen.width_in_pixels, screen.height_in_pixels);
    let image = conn.get_image(ImageFormat::Z_PIXMAP, screen.root, 0, 0, w, h, !0)?.reply()?;
    if image.depth != 24 && image.depth != 32 {
        anyhow::bail!("an X11 screen of depth {} can't be captured", image.depth);
    }
    // ZPixmap at depth 24/32 is 4 bytes a pixel, blue first (little-endian), whatever the visual's masks.
    let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
    for px in image.data.chunks_exact(4) {
        rgba.extend_from_slice(&[px[2], px[1], px[0], 255]);
    }
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut encoder = png::Encoder::new(file, u32::from(w), u32::from(h));
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&rgba)?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn wayland_tool(path: &std::path::Path) -> Result<Option<PathBuf>> {
    use std::process::Command;
    let p = path.to_string_lossy().into_owned();
    let tools: [(&str, Vec<&str>); 3] = [
        ("grim", vec![p.as_str()]),
        ("gnome-screenshot", vec!["-f", p.as_str()]),
        ("spectacle", vec!["-b", "-n", "-f", "-o", p.as_str()]),
    ];
    for (tool, args) in tools {
        if let Ok(status) = Command::new(tool).args(&args).status() {
            if status.success() && path.exists() {
                return Ok(Some(path.to_path_buf()));
            }
        }
    }
    anyhow::bail!("no screenshot tool found for Wayland (grim, gnome-screenshot or spectacle)")
}

/// On Windows, PowerShell copies the whole virtual screen (every monitor) and saves it as a PNG.
#[cfg(target_os = "windows")]
pub fn capture() -> Result<Option<PathBuf>> {
    use std::process::Command;
    crate::paths::ensure_dirs()?;
    let path = crate::paths::screenshots_dir().join(format!("screenshot-{}.png", crate::util::now_millis()));
    let script = format!(
        "Add-Type -AssemblyName System.Windows.Forms,System.Drawing; \
         $b=[System.Windows.Forms.SystemInformation]::VirtualScreen; \
         $bmp=New-Object System.Drawing.Bitmap $b.Width,$b.Height; \
         $g=[System.Drawing.Graphics]::FromImage($bmp); \
         $g.CopyFromScreen($b.Left,$b.Top,0,0,$bmp.Size); \
         $bmp.Save('{}',[System.Drawing.Imaging.ImageFormat]::Png)",
        path.display().to_string().replace('\'', "''")
    );
    let status = Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &script]).status()?;
    if !status.success() || !path.exists() {
        anyhow::bail!("the screen capture failed (PowerShell exited with {:?})", status.code());
    }
    Ok(Some(path))
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn capture() -> Result<Option<PathBuf>> {
    anyhow::bail!("screen capture is not available on this OS")
}
