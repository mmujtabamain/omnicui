//! The gpui target's adapter. omnicui starts it once per check and sends JSON-RPC 2.0 requests,
//! one per line, on standard input; every reply is one line on standard output. Logs go to
//! standard error only.
//!
//! One long-lived GPUI application. Each `mount` loads its fixture into a fresh data folder
//! (the fixture loader, through the app's own Store), then opens a real GPUI window at the
//! viewport's size with the app's surface in it (components.rs). On Linux the windows live on
//! an X server: Xvfb, started here when there is no DISPLAY, drawn by GPUI's Vulkan renderer
//! (Mesa's lavapipe where there is no GPU). Input is real: XTEST pointer and key events, which
//! reach GPUI through its own X11 platform code. Parts are the app's part tags (taskscape_gpui::
//! parts), recorded with their boxes at every frame. Screenshots read the window's pixels from
//! the X server. Events are what the app reports as the user changes things.

mod components;
mod xlib;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};
use gpui::{
    div, point, prelude::*, px, rgb, size, AnyWindowHandle, App, Application, AsyncApp, Bounds, WindowBackgroundAppearance,
    WindowBounds, WindowDecorations, WindowKind, WindowOptions,
};
use serde_json::{json, Value};
use taskscape_gpui::{motion, parts, Assets, Db};

use components::Root;

const NO_COMPONENT: i64 = -32001;
const NO_PART: i64 = -32002;
const STEP_FAILED: i64 = -32003;
const NO_METHOD: i64 = -32601;

struct RpcErr(i64, String);
impl From<anyhow::Error> for RpcErr {
    fn from(e: anyhow::Error) -> Self {
        RpcErr(STEP_FAILED, format!("{e:#}"))
    }
}
type R<T> = std::result::Result<T, RpcErr>;

fn log(msg: impl AsRef<str>) {
    eprintln!("[gpui] {}", msg.as_ref());
}

struct Logger;
impl log::Log for Logger {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Warn && !m.target().starts_with("naga")
    }
    fn log(&self, r: &log::Record) {
        if self.enabled(r.metadata()) {
            eprintln!("[gpui] {} {}: {}", r.level(), r.target(), r.args());
        }
    }
    fn flush(&self) {}
}
static LOGGER: Logger = Logger;

/// A mounted surface.
struct Mounted {
    component: String,
    window: AnyWindowHandle,
    xid: xlib::Window,
    origin: (i32, i32),
    viewport: (u32, u32),
    _root: Root,
    _db: Rc<Db>,
    home: PathBuf,
    /// Store id → fixture key.
    names: HashMap<String, String>,
}

struct Adapter {
    x: xlib::X,
    rt: Arc<tokio::runtime::Runtime>,
    data: PathBuf,
    mounted: Option<Mounted>,
    events: Rc<RefCell<Vec<(String, Value)>>>,
    serial: u64,
    started: Instant,
}

fn main() {
    let started = Instant::now();
    log::set_logger(&LOGGER).ok();
    log::set_max_level(log::LevelFilter::Warn);
    let xvfb = match ensure_display() {
        Ok(x) => x,
        Err(e) => {
            log(format!("no X display: {e:#}"));
            std::process::exit(1);
        }
    };
    let x = match xlib::X::open() {
        Ok(x) => x,
        Err(e) => {
            log(format!("{e:#}"));
            std::process::exit(1);
        }
    };
    let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().expect("tokio"));
    // Fixtures are written into memory where the system has a memory file system: SQLite's
    // commits then cost nothing, and a mount doesn't wait on the disk.
    let tmp = if Path::new("/dev/shm").is_dir() { PathBuf::from("/dev/shm") } else { std::env::temp_dir() };
    let data = tmp.join(format!("omnicui-gpui-{}", std::process::id()));
    std::fs::create_dir_all(&data).ok();

    // Requests come in on a thread of their own and are handled on GPUI's main thread.
    let (tx, rx) = mpsc::channel::<Option<String>>();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) if !l.trim().is_empty() => {
                    if tx.send(Some(l)).is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        let _ = tx.send(None);
    });

    let xvfb = Rc::new(RefCell::new(xvfb));
    let xvfb2 = xvfb.clone();
    Application::new().with_assets(Assets).run(move |cx: &mut App| {
        taskscape_gpui::init(cx);
        parts::enable();
        let events: Rc<RefCell<Vec<(String, Value)>>> = Rc::default();
        let ev = events.clone();
        taskscape_gpui::set_reporter(cx, Rc::new(move |name, fields| ev.borrow_mut().push((name.to_string(), fields))));
        // GPUI's X11 platform ends the event loop when its last window closes: a small window in
        // the screen's far corner, away from every mount, keeps it running between mounts.
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(1580.), px(980.)), size: size(px(8.), px(8.)) })),
                titlebar: None,
                focus: false,
                show: true,
                ..Default::default()
            },
            |_, cx| cx.new(|_| Blank),
        )
        .ok();
        let adapter = Rc::new(RefCell::new(Adapter { x, rt, data, mounted: None, events, serial: 0, started }));
        cx.spawn(async move |cx| {
            loop {
                let line = match rx.try_recv() {
                    Ok(Some(l)) => l,
                    Ok(None) | Err(mpsc::TryRecvError::Disconnected) => break,
                    Err(mpsc::TryRecvError::Empty) => {
                        cx.background_executor().timer(Duration::from_millis(2)).await;
                        continue;
                    }
                };
                let reply = handle(&adapter, &line, cx).await;
                let mut out = std::io::stdout().lock();
                let _ = writeln!(out, "{reply}");
                let _ = out.flush();
            }
            // Standard input closed: clean up and leave.
            let mut a = adapter.borrow_mut();
            if let Some(m) = a.mounted.take() {
                let _ = std::fs::remove_dir_all(&m.home);
            }
            let _ = std::fs::remove_dir_all(&a.data);
            if let Some(mut child) = xvfb2.borrow_mut().take() {
                let _ = child.kill();
            }
            cx.update(|cx| cx.quit()).ok();
            std::process::exit(0);
        })
        .detach();
    });
    let child = xvfb.borrow_mut().take();
    if let Some(mut child) = child {
        let _ = child.kill();
    }
}

/// Uses `DISPLAY` if set; otherwise starts Xvfb on a free display number.
fn ensure_display() -> Result<Option<Child>> {
    if cfg!(not(target_os = "linux")) || std::env::var("DISPLAY").is_ok_and(|d| !d.is_empty()) {
        return Ok(None);
    }
    for n in 140..400 {
        let (sock, lock) = (format!("/tmp/.X11-unix/X{n}"), format!("/tmp/.X{n}-lock"));
        // A server that was killed (omnicui stops a target's whole process group) leaves its lock
        // and socket behind: a lock whose process is gone is free again.
        if let Ok(pid) = std::fs::read_to_string(&lock) {
            let alive = pid.trim().parse::<u32>().is_ok_and(|p| Path::new(&format!("/proc/{p}")).exists());
            if alive {
                continue;
            }
            let _ = std::fs::remove_file(&lock);
            let _ = std::fs::remove_file(&sock);
        } else if Path::new(&sock).exists() {
            continue;
        }
        let mut cmd = Command::new("Xvfb");
        // The server goes with the adapter, however the adapter ends (a panic included).
        unsafe {
            use std::os::unix::process::CommandExt;
            cmd.pre_exec(|| {
                libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
                Ok(())
            });
        }
        let child = cmd
            .args([&format!(":{n}"), "-screen", "0", "1600x1000x24", "-nolisten", "tcp"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| anyhow!("Xvfb can't be started ({e}); install it, or set DISPLAY to an X server"))?;
        let until = Instant::now() + Duration::from_secs(10);
        while Instant::now() < until {
            if Path::new(&format!("/tmp/.X11-unix/X{n}")).exists() {
                std::env::set_var("DISPLAY", format!(":{n}"));
                return Ok(Some(child));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        bail!("Xvfb :{n} didn't come up in 10 s");
    }
    bail!("no free X display number")
}

async fn handle(a: &Rc<RefCell<Adapter>>, line: &str, cx: &mut AsyncApp) -> String {
    let req: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(_) => return json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "parse error"}}).to_string(),
    };
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("").to_string();
    let p = req.get("params").cloned().unwrap_or(json!({}));
    let result = dispatch(a, &method, &p, cx).await;
    match result {
        Ok(v) => json!({"jsonrpc": "2.0", "id": id, "result": v}).to_string(),
        Err(RpcErr(code, msg)) => {
            log(format!("{method} failed: {msg}"));
            json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": msg}}).to_string()
        }
    }
}

fn sp(p: &Value, k: &str) -> String {
    p.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

/// `taskscape.pane.row3` → `row3`
fn part_name(full: &str) -> String {
    match full.strip_prefix("taskscape.") {
        Some(rest) => rest.split_once('.').map(|(_, p)| p.to_string()).unwrap_or_else(|| rest.to_string()),
        None => full.to_string(),
    }
}

async fn dispatch(a: &Rc<RefCell<Adapter>>, method: &str, p: &Value, cx: &mut AsyncApp) -> R<Value> {
    match method {
        "advertise" => {
            log(format!("ready in {} ms (an X server, GPUI on Vulkan)", a.borrow().started.elapsed().as_millis()));
            Ok(json!({
                "protocol": "1",
                "can": ["mount", "press", "focus", "key", "type", "drag", "hover", "read", "events", "screenshot", "trace", "trace.opacity", "record", "probe", "bounds", "role", "focused", "selected", "do.summon"],
                "cannot": {
                    "trace.scale": "GPUI has no transform for a box: the app draws a scale-in as the box growing and fading, which a scale trace can't read",
                    "do.back": "a desktop window has no system back",
                    "do.scroll": "not implemented in this target",
                    "do.swipe": "a desktop window has no swipe",
                    "do.refresh": "the app has no pull to refresh",
                    "do.pick": "the app has no picker",
                },
                "keys": { "button": ["Enter", "Space"], "switch": ["Space"], "checkbox": ["Space"] },
                "platform": { "os": std::env::consts::OS.replace("macos", "macos") },
            }))
        }
        "probe" => probe(a, p, cx).await,
        "mount" => mount(a, p, cx).await,
        "press" | "hover" => {
            let (x, y) = center(a, &part_name(&sp(p, "part")))?;
            {
                let ad = a.borrow();
                ad.x.move_to(x, y);
                if method == "press" {
                    ad.x.button(1, true);
                    ad.x.button(1, false);
                }
            }
            settle(a, cx).await;
            Ok(Value::Null)
        }
        "focus" => {
            focus_part(a, &part_name(&sp(p, "part")), cx)?;
            settle(a, cx).await;
            Ok(Value::Null)
        }
        "key" => {
            let part = sp(p, "part");
            if !part.is_empty() {
                find(&part_name(&part))?;
            }
            send_key(a, &sp(p, "key"))?;
            settle(a, cx).await;
            Ok(Value::Null)
        }
        "type" => {
            focus_part(a, &part_name(&sp(p, "part")), cx)?;
            settle(a, cx).await;
            type_text(a, &sp(p, "text"))?;
            settle(a, cx).await;
            Ok(Value::Null)
        }
        "drag" => {
            let (x, y) = center(a, &part_name(&sp(p, "part")))?;
            let dx = p.get("dx").and_then(|v| v.as_f64()).unwrap_or(0.) as i32;
            let dy = p.get("dy").and_then(|v| v.as_f64()).unwrap_or(0.) as i32;
            {
                let ad = a.borrow();
                ad.x.move_to(x, y);
                ad.x.button(1, true);
            }
            for i in 1..=10 {
                a.borrow().x.move_to(x + dx * i / 10, y + dy * i / 10);
                cx.background_executor().timer(Duration::from_millis(16)).await;
            }
            a.borrow().x.button(1, false);
            settle(a, cx).await;
            Ok(Value::Null)
        }
        "read" => read(a, &part_name(&sp(p, "part")), &sp(p, "field"), cx),
        "bounds" => {
            let (info, _) = find(&part_name(&sp(p, "part")))?;
            let b = info.bounds;
            Ok(json!({"x": f32::from(b.origin.x), "y": f32::from(b.origin.y), "width": f32::from(b.size.width), "height": f32::from(b.size.height)}))
        }
        "events" => {
            let ad = a.borrow();
            let names = ad.mounted.as_ref().map(|m| m.names.clone()).unwrap_or_default();
            let out: Vec<Value> = ad.events.borrow_mut().drain(..).map(|(n, f)| json!({"name": n, "fields": by_key(f, &names)})).collect();
            Ok(Value::Array(out))
        }
        "screenshot" => {
            settle(a, cx).await;
            let path = sp(p, "path");
            capture_to(a, Path::new(&path))?;
            Ok(json!({"path": path, "regions": regions(a)}))
        }
        // The capture bar's global shortcut (⌘↩), which the system delivers to the app.
        "do" => {
            if sp(p, "name") != "summon" {
                return Err(RpcErr(NO_METHOD, format!("do.{} is not something this target does", sp(p, "name"))));
            }
            let (w, bar) = {
                let ad = a.borrow();
                let m = ad.mounted.as_ref().ok_or_else(|| RpcErr(NO_COMPONENT, "nothing mounted".into()))?;
                match &m._root {
                    Root::Capture(b) => (m.window, b.clone()),
                    _ => return Err(RpcErr(STEP_FAILED, "summon is the capture bar's".into())),
                }
            };
            cx.update_window(w, |_, window, cx| bar.update(cx, |b, cx| b.summon(window, cx))).map_err(|e| RpcErr(STEP_FAILED, format!("{e}")))?;
            settle(a, cx).await;
            Ok(Value::Null)
        }
        "trace" => trace(a, p, cx).await,
        "record" => record(a, p, cx).await,
        other => Err(RpcErr(NO_METHOD, format!("unknown method {other}"))),
    }
}

/// Ids in an event's fields, as the fixture's keys.
fn by_key(fields: Value, names: &HashMap<String, String>) -> Value {
    let Value::Object(mut m) = fields else { return fields };
    for k in ["id", "listId", "parentId", "taskId", "projectId"] {
        if let Some(Value::String(v)) = m.get(k) {
            if let Some(key) = names.get(v) {
                m.insert(k.into(), Value::String(key.clone()));
            }
        }
    }
    if let Some(Value::String(c)) = m.get("command").cloned() {
        for pre in ["go-list-", "go-proj-"] {
            if let Some(id) = c.strip_prefix(pre) {
                if let Some(key) = names.get(id) {
                    m.insert("command".into(), Value::String(format!("{pre}{key}")));
                }
            }
        }
    }
    Value::Object(m)
}

fn find(part: &str) -> R<(parts::PartInfo, Option<gpui::FocusHandle>)> {
    parts::get(part).ok_or_else(|| RpcErr(NO_PART, format!("no part {part}")))
}

/// The middle of a part's visible box, on the screen.
fn center(a: &Rc<RefCell<Adapter>>, part: &str) -> R<(i32, i32)> {
    let (info, _) = find(part)?;
    let b = if info.visible.size.width > px(0.) && info.visible.size.height > px(0.) { info.visible } else { info.bounds };
    if b.size.width <= px(0.) || b.size.height <= px(0.) {
        return Err(RpcErr(STEP_FAILED, format!("{part} has no size on screen")));
    }
    let ad = a.borrow();
    let (ox, oy) = ad.mounted.as_ref().map(|m| m.origin).unwrap_or((0, 0));
    let c = b.center();
    Ok((ox + f32::from(c.x).round() as i32, oy + f32::from(c.y).round() as i32))
}

fn focus_part(a: &Rc<RefCell<Adapter>>, part: &str, cx: &mut AsyncApp) -> R<()> {
    let (_, focus) = find(part)?;
    let Some(focus) = focus else { return Err(RpcErr(STEP_FAILED, format!("{part} can't take keyboard focus"))) };
    let w = a.borrow().mounted.as_ref().map(|m| m.window).ok_or_else(|| RpcErr(NO_COMPONENT, "nothing mounted".into()))?;
    cx.update_window(w, |_, window, _| window.focus(&focus)).map_err(|e| RpcErr(STEP_FAILED, format!("{e}")))?;
    Ok(())
}

fn read(a: &Rc<RefCell<Adapter>>, part: &str, field: &str, cx: &mut AsyncApp) -> R<Value> {
    let (info, focus) = find(part)?;
    Ok(match field {
        "text" => json!(info.text.unwrap_or_default()),
        "name" => json!(info.name.or(info.text)),
        "checked" => info.checked.map(Value::Bool).unwrap_or(Value::Null),
        "disabled" => json!(info.disabled),
        "selected" => json!(info.selected.unwrap_or(false)),
        "value" => info.value.map(|v| json!(v)).unwrap_or(Value::Null),
        "role" => json!(info.role),
        "focused" => {
            let w = a.borrow().mounted.as_ref().map(|m| m.window);
            match (focus, w) {
                (Some(f), Some(w)) => json!(cx.update_window(w, |_, window, _| f.is_focused(window)).unwrap_or(false)),
                _ => json!(false),
            }
        }
        other => return Err(RpcErr(STEP_FAILED, format!("unknown field {other}"))),
    })
}

// ---- keys -------------------------------------------------------------------------------------

fn keysym(name: &str) -> String {
    match name {
        "Meta" => "Super_L",
        "Alt" => "Alt_L",
        "Shift" => "Shift_L",
        "Control" => "Control_L",
        "Enter" => "Return",
        "ArrowDown" => "Down",
        "ArrowUp" => "Up",
        "ArrowLeft" => "Left",
        "ArrowRight" => "Right",
        "Backspace" => "BackSpace",
        "Space" => "space",
        "Backslash" => "backslash",
        "Comma" => "comma",
        "Period" => "period",
        "Slash" => "slash",
        "Minus" => "minus",
        "Equal" => "equal",
        "BracketLeft" => "bracketleft",
        "BracketRight" => "bracketright",
        other => return other.to_string(),
    }
    .to_string()
}

fn send_key(a: &Rc<RefCell<Adapter>>, combo: &str) -> R<()> {
    let ad = a.borrow();
    let parts: Vec<&str> = combo.split('+').collect();
    let (mods, key) = parts.split_at(parts.len() - 1);
    let mut held = vec![];
    for m in mods {
        let (code, _) = ad.x.key_for(&keysym(m)).map_err(RpcErr::from)?;
        ad.x.key(code, true);
        held.push(code);
    }
    let name = keysym(key[0]);
    let name = if name.len() == 1 { name.to_lowercase() } else { name };
    let (code, shifted) = ad.x.key_for(&name).map_err(RpcErr::from)?;
    let shift = if shifted && !mods.contains(&"Shift") { Some(ad.x.key_for("Shift_L").map_err(RpcErr::from)?.0) } else { None };
    if let Some(s) = shift {
        ad.x.key(s, true);
    }
    ad.x.key(code, true);
    ad.x.key(code, false);
    if let Some(s) = shift {
        ad.x.key(s, false);
    }
    for c in held.into_iter().rev() {
        ad.x.key(c, false);
    }
    ad.x.sync();
    Ok(())
}

fn char_keysym(c: char) -> R<String> {
    Ok(match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' => c.to_string(),
        ' ' => "space".into(),
        '.' => "period".into(),
        ',' => "comma".into(),
        '-' => "minus".into(),
        '_' => "underscore".into(),
        '/' => "slash".into(),
        ':' => "colon".into(),
        ';' => "semicolon".into(),
        '\'' => "apostrophe".into(),
        '"' => "quotedbl".into(),
        '!' => "exclam".into(),
        '?' => "question".into(),
        '(' => "parenleft".into(),
        ')' => "parenright".into(),
        '&' => "ampersand".into(),
        '@' => "at".into(),
        '#' => "numbersign".into(),
        '*' => "asterisk".into(),
        '+' => "plus".into(),
        '=' => "equal".into(),
        '`' => "grave".into(),
        '\n' => "Return".into(),
        other => return Err(RpcErr(STEP_FAILED, format!("this target's keyboard can't type {other:?}"))),
    })
}

fn type_text(a: &Rc<RefCell<Adapter>>, text: &str) -> R<()> {
    let ad = a.borrow();
    let shift = ad.x.key_for("Shift_L").map_err(RpcErr::from)?.0;
    for c in text.chars() {
        let (code, shifted) = ad.x.key_for(&char_keysym(c)?).map_err(RpcErr::from)?;
        if shifted {
            ad.x.key(shift, true);
        }
        ad.x.key(code, true);
        ad.x.key(code, false);
        if shifted {
            ad.x.key(shift, false);
        }
    }
    ad.x.sync();
    Ok(())
}

// ---- settling ----------------------------------------------------------------------------------

/// Waits until the app has handled the input and drawn what it changed: no animation running and
/// no new frame for 48 ms (GPUI draws only when something changed), at most 10 s.
async fn settle(a: &Rc<RefCell<Adapter>>, cx: &mut AsyncApp) {
    a.borrow().x.sync();
    let until = Instant::now() + Duration::from_secs(10);
    cx.background_executor().timer(Duration::from_millis(24)).await;
    loop {
        let p0 = parts::paints();
        let anim0 = motion::take_animating();
        cx.background_executor().timer(Duration::from_millis(48)).await;
        let anim1 = motion::take_animating();
        if parts::paints() == p0 && !anim0 && !anim1 {
            break;
        }
        if Instant::now() > until {
            log("still drawing after 10 s");
            break;
        }
    }
}

/// Waits until a frame drawn now is on screen: the renderer presents a frame after GPUI has
/// painted it, so a picture taken right after a paint can still show the frame before. A second
/// frame drawn after the first means the first has been presented.
async fn presented_frame(a: &Rc<RefCell<Adapter>>, cx: &mut AsyncApp) {
    next_frame(a, cx).await;
    next_frame(a, cx).await;
}

/// Waits for the next frame drawn (after a refresh), up to a second.
async fn next_frame(a: &Rc<RefCell<Adapter>>, cx: &mut AsyncApp) {
    let w = a.borrow().mounted.as_ref().map(|m| m.window);
    let p0 = parts::paints();
    if let Some(w) = w {
        cx.update_window(w, |_, window, _| window.refresh()).ok();
    }
    let until = Instant::now() + Duration::from_secs(1);
    while parts::paints() == p0 && Instant::now() < until {
        cx.background_executor().timer(Duration::from_millis(2)).await;
    }
    a.borrow().x.sync();
}

// ---- mount -------------------------------------------------------------------------------------

fn close_mounted(a: &Rc<RefCell<Adapter>>, cx: &mut AsyncApp) {
    let old = a.borrow_mut().mounted.take();
    if let Some(m) = old {
        cx.update_window(m.window, |_, window, _| window.remove_window()).ok();
        let home = m.home.clone();
        drop(m);
        let _ = std::fs::remove_dir_all(home);
    }
}

async fn open_window(a: &Rc<RefCell<Adapter>>, w: u32, h: u32, cx: &mut AsyncApp, build: impl FnOnce(&mut gpui::Window, &mut App) -> gpui::AnyView + 'static) -> R<(AnyWindowHandle, String)> {
    let title = {
        let mut ad = a.borrow_mut();
        ad.serial += 1;
        format!("omnicui-gpui-mount-{}-{}", std::process::id(), ad.serial)
    };
    let bounds = Bounds { origin: point(px(0.), px(0.)), size: size(px(w as f32), px(h as f32)) };
    let t2 = title.clone();
    let handle = cx
        .update(|cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions { title: Some(t2.into()), appears_transparent: true, traffic_light_position: None }),
                    window_decorations: Some(WindowDecorations::Client),
                    kind: WindowKind::Normal,
                    focus: true,
                    show: true,
                    is_resizable: false,
                    window_background: WindowBackgroundAppearance::Opaque,
                    ..Default::default()
                },
                |window, cx| {
                    let view = build(window, cx);
                    cx.new(|_| Host(view))
                },
            )
        })
        .map_err(|e| RpcErr(NO_COMPONENT, format!("{e}")))?
        .map_err(|e| RpcErr(NO_COMPONENT, format!("the window can't be opened: {e:#}")))?;
    Ok((handle.into(), title))
}

async fn find_window(a: &Rc<RefCell<Adapter>>, title: &str, cx: &mut AsyncApp) -> R<(xlib::Window, (i32, i32))> {
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(w) = a.borrow().x.find(title) {
            let ad = a.borrow();
            ad.x.focus(w);
            return Ok((w, ad.x.origin(w)));
        }
        if Instant::now() > until {
            return Err(RpcErr(NO_COMPONENT, "the window never appeared on the X server".into()));
        }
        cx.background_executor().timer(Duration::from_millis(10)).await;
    }
}

async fn mount(a: &Rc<RefCell<Adapter>>, p: &Value, cx: &mut AsyncApp) -> R<Value> {
    let component = sp(p, "component");
    let vw = p.pointer("/viewport/width").and_then(|v| v.as_u64()).unwrap_or(800) as u32;
    let vh = p.pointer("/viewport/height").and_then(|v| v.as_u64()).unwrap_or(600) as u32;
    let dark = sp(p, "colorScheme") == "dark";
    if !components::COMPONENTS.contains(&component.as_str()) {
        return Err(RpcErr(NO_COMPONENT, format!("no component {component}")));
    }
    let t0 = Instant::now();
    close_mounted(a, cx);
    motion::release();
    let props = components::with_defaults(&component, p.get("props").unwrap_or(&Value::Null));
    let seed = props.get("seed").and_then(|v| v.as_str()).unwrap_or("standard").to_string();

    // A fresh data folder with the fixture in it, through the app's own Store.
    let (rt, data) = {
        let ad = a.borrow();
        (ad.rt.clone(), ad.data.clone())
    };
    let loaded = rt.block_on(taskscape_fixtures::load(&seed, &data)).map_err(|e| RpcErr(NO_COMPONENT, format!("fixture {seed}: {e:#}")))?;
    log(format!("mount: fixture {} ms", t0.elapsed().as_millis()));
    let keys: BTreeMap<String, String> = loaded.keys.clone();
    let names: HashMap<String, String> = keys.iter().map(|(k, id)| (id.clone(), k.clone())).collect();
    let home = loaded.home.clone();
    let db = Rc::new(Db::new(loaded.store, rt.clone()));
    // Settings the props stand for, saved before the app reads them, as the app saves them.
    if props.get("showCompleted").and_then(|v| v.as_bool()) == Some(false) {
        db.set_setting("show_completed", "0");
    }
    cx.update(|cx| keys_rebind(cx, &db)).ok();

    let root: Rc<RefCell<Option<Result<Root, String>>>> = Rc::default();
    let (root2, db2, comp2) = (root.clone(), db.clone(), component.clone());
    let lookup = move |k: &str| keys.get(k).cloned();
    let (window, title) = open_window(a, vw, vh, cx, move |window, cx| {
        let built = components::build(&comp2, &props, db2, dark, &lookup, window, cx);
        let view: gpui::AnyView = match &built {
            Ok(Root::Ws(v)) => v.clone().into(),
            Ok(Root::Settings(v)) => v.clone().into(),
            Ok(Root::Capture(v)) => v.clone().into(),
            Err(_) => cx.new(|_| Blank).into(),
        };
        *root2.borrow_mut() = Some(built);
        view
    })
    .await?;
    let built = root.borrow_mut().take().unwrap_or_else(|| Err("nothing was built".into()));
    let root = match built {
        Ok(r) => r,
        Err(e) => {
            cx.update_window(window, |_, w, _| w.remove_window()).ok();
            let _ = std::fs::remove_dir_all(&home);
            return Err(RpcErr(NO_COMPONENT, e));
        }
    };
    log(format!("mount: window opened {} ms", t0.elapsed().as_millis()));
    let (xid, origin) = find_window(a, &title, cx).await?;
    log(format!("mount: window found {} ms", t0.elapsed().as_millis()));
    {
        let ad = a.borrow();
        // The pointer waits outside the window, so nothing starts out hovered.
        ad.x.move_to(1599, 999);
    }
    a.borrow_mut().mounted = Some(Mounted { component, window, xid, origin, viewport: (vw, vh), _root: root, _db: db, home, names });
    next_frame(a, cx).await;
    settle(a, cx).await;
    a.borrow().events.borrow_mut().clear();
    log(format!("mount: settled {} ms", t0.elapsed().as_millis()));
    Ok(Value::Null)
}

fn keys_rebind(cx: &mut App, db: &Db) {
    taskscape_gpui::keys::rebind(cx, &db.hotkeys());
}

/// The window's root: the surface the registry built.
struct Host(gpui::AnyView);
impl Render for Host {
    fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div().size_full().child(self.0.clone())
    }
}

struct Blank;
impl Render for Blank {
    fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
    }
}

// ---- pictures ----------------------------------------------------------------------------------

fn capture_rgba(a: &Rc<RefCell<Adapter>>) -> R<(Vec<u8>, u32, u32)> {
    let ad = a.borrow();
    let m = ad.mounted.as_ref().ok_or_else(|| RpcErr(NO_COMPONENT, "nothing mounted".into()))?;
    let (w, h) = m.viewport;
    Ok((ad.x.capture(m.xid, w, h).map_err(RpcErr::from)?, w, h))
}

fn capture_to(a: &Rc<RefCell<Adapter>>, path: &Path) -> R<()> {
    let (px_, w, h) = capture_rgba(a)?;
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).ok();
    }
    image::save_buffer(path, &px_, w, h, image::ColorType::Rgba8).map_err(|e| RpcErr(STEP_FAILED, format!("{e}")))?;
    Ok(())
}

/// The window chrome a native window would draw, as the tauri target reports it: the app draws
/// its own title bar, so on the whole window and on Settings the bar is reported whole (omnicui
/// cuts it off), and elsewhere only the window controls are (omnicui masks them).
fn regions(a: &Rc<RefCell<Adapter>>) -> Value {
    let comp = a.borrow().mounted.as_ref().map(|m| m.component.clone()).unwrap_or_default();
    let boxed = |part: &str| {
        parts::get(part).filter(|(i, _)| i.bounds.size.width > px(0.)).map(|(i, _)| {
            let b = i.bounds;
            json!({"kind": "titlebar", "x": f32::from(b.origin.x).round(), "y": f32::from(b.origin.y).round(), "width": f32::from(b.size.width).round(), "height": f32::from(b.size.height).round()})
        })
    };
    let whole = match comp.as_str() {
        "taskscape.window" => parts::get("windowControls").and(None).or_else(|| titlebar_box()),
        "taskscape.settings" => boxed("settingsTitlebar"),
        _ => None,
    };
    let list: Vec<Value> = match whole {
        Some(w) => vec![w],
        None => boxed("windowControls").into_iter().collect(),
    };
    Value::Array(list)
}

/// The main window's title bar: the full width, from the top to the bar's bottom edge.
fn titlebar_box() -> Option<Value> {
    let (p, _) = parts::get("project")?;
    let bottom = 46.0f32.max(f32::from(p.bounds.origin.y + p.bounds.size.height));
    Some(json!({"kind": "titlebar", "x": 0, "y": 0, "width": 1180, "height": bottom.round()}))
}

/// The render check's picture: white, a black 32×32 square at 8,8 and a red one 8 px from the
/// bottom-right corner, drawn by GPUI in a window of the size asked for.
async fn probe(a: &Rc<RefCell<Adapter>>, p: &Value, cx: &mut AsyncApp) -> R<Value> {
    let w = p.pointer("/viewport/width").and_then(|v| v.as_u64()).unwrap_or(800) as u32;
    let h = p.pointer("/viewport/height").and_then(|v| v.as_u64()).unwrap_or(600) as u32;
    close_mounted(a, cx);
    let (window, title) = open_window(a, w, h, cx, |_, cx| cx.new(|_| Probe).into()).await?;
    let (xid, origin) = find_window(a, &title, cx).await?;
    // Two frames, so the first one is on screen.
    for _ in 0..3 {
        cx.update_window(window, |_, w, _| w.refresh()).ok();
        cx.background_executor().timer(Duration::from_millis(60)).await;
    }
    let path = sp(p, "path");
    let pixels = a.borrow().x.capture(xid, w, h).map_err(RpcErr::from)?;
    let _ = origin;
    if let Some(d) = Path::new(&path).parent() {
        std::fs::create_dir_all(d).ok();
    }
    image::save_buffer(&path, &pixels, w, h, image::ColorType::Rgba8).map_err(|e| RpcErr(STEP_FAILED, format!("{e}")))?;
    cx.update_window(window, |_, w, _| w.remove_window()).ok();
    Ok(json!(path))
}

struct Probe;
impl Render for Probe {
    fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .relative()
            .bg(rgb(0xffffff))
            .child(div().absolute().left(px(8.)).top(px(8.)).size(px(32.)).bg(rgb(0x000000)))
            .child(div().absolute().right(px(8.)).bottom(px(8.)).size(px(32.)).bg(rgb(0xff0000)))
    }
}

// ---- motion ------------------------------------------------------------------------------------

async fn run_steps(a: &Rc<RefCell<Adapter>>, steps: &[Value], settled: bool, cx: &mut AsyncApp) -> R<()> {
    for s in steps {
        let m = sp(s, "method");
        if settled {
            Box::pin(dispatch(a, &m, s, cx)).await?;
        } else {
            step_unsettled(a, &m, s, cx)?;
        }
    }
    Ok(())
}

/// A step's input, without waiting for what it starts.
fn step_unsettled(a: &Rc<RefCell<Adapter>>, m: &str, s: &Value, cx: &mut AsyncApp) -> R<()> {
    match m {
        "press" | "hover" => {
            let (x, y) = center(a, &part_name(&sp(s, "part")))?;
            let ad = a.borrow();
            ad.x.move_to(x, y);
            if m == "press" {
                ad.x.button(1, true);
                ad.x.button(1, false);
            }
            ad.x.sync();
            Ok(())
        }
        "key" => send_key(a, &sp(s, "key")),
        "type" => {
            focus_part(a, &part_name(&sp(s, "part")), cx)?;
            type_text(a, &sp(s, "text"))
        }
        "focus" => focus_part(a, &part_name(&sp(s, "part")), cx),
        other => Err(RpcErr(STEP_FAILED, format!("{other} can't be recorded here"))),
    }
}

/// Samples the part at every frame drawn while the steps run. As for clips, the app's animation
/// clock is held and moved on 16 ms per frame, so the samples are the motion as the app computes
/// it, not the software renderer's frame rate (llvmpipe draws a 1180×760 frame in 15–40 ms).
async fn trace(a: &Rc<RefCell<Adapter>>, p: &Value, cx: &mut AsyncApp) -> R<Value> {
    let part = part_name(&sp(p, "part"));
    let property = sp(p, "property");
    if property == "scale" {
        return Err(RpcErr(STEP_FAILED, "scale can't be traced in GPUI (no transforms)".into()));
    }
    let center_point = sp(p, "point") == "center";
    let steps = p.get("steps").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    settle(a, cx).await;
    motion::hold();
    parts::trace_start(&part);
    let r: R<()> = async {
        // The part at rest first: GPUI draws nothing while nothing changes.
        for _ in 0..2 {
            motion::step(Duration::from_millis(16));
            next_frame(a, cx).await;
        }
        for s in &steps {
            step_unsettled(a, &sp(s, "method"), s, cx)?;
            cx.background_executor().timer(Duration::from_millis(40)).await;
            next_frame(a, cx).await;
            let mut quiet = 0;
            for _ in 0..60 {
                motion::step(Duration::from_millis(16));
                motion::take_animating();
                next_frame(a, cx).await;
                if motion::take_animating() {
                    quiet = 0;
                } else {
                    quiet += 1;
                    if quiet >= 3 {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
    .await;
    let samples = parts::trace_take();
    motion::release();
    settle(a, cx).await;
    r?;
    if samples.is_empty() {
        return Err(RpcErr(NO_PART, format!("no part {part} (not on screen at any frame)")));
    }
    let t0 = samples[0].at;
    let out: Vec<Value> = samples
        .iter()
        .map(|s| {
            let ms = (s.at.saturating_sub(t0)).as_secs_f64() * 1000.0;
            if property == "opacity" {
                json!({"ms": ms, "x": 0, "y": 0, "value": s.opacity})
            } else {
                let b = s.bounds;
                let (x, y) = if center_point { (b.center().x, b.center().y) } else { (b.origin.x, b.origin.y) };
                json!({"ms": ms, "x": f32::from(x), "y": f32::from(y)})
            }
        })
        .collect();
    Ok(Value::Array(out))
}

/// A clip on the app's animation clock: the clock is held, and moved on 16 ms per frame, with a
/// picture of the window at each. A 110 ms rise is seven frames whatever the machine's load.
async fn record(a: &Rc<RefCell<Adapter>>, p: &Value, cx: &mut AsyncApp) -> R<Value> {
    let dir = PathBuf::from(sp(p, "path"));
    std::fs::create_dir_all(&dir).ok();
    let steps = p.get("steps").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let mut frames = vec![];
    let mut inputs = vec![];
    let mut ms = 0u64;
    settle(a, cx).await;
    motion::hold();
    let result: R<()> = async {
        let mut shot = |a: &Rc<RefCell<Adapter>>, frames: &mut Vec<Value>, ms: &mut u64| -> R<()> {
            let file = dir.join(format!("{}.png", frames.len()));
            capture_to(a, &file)?;
            frames.push(json!({"ms": *ms, "file": file.display().to_string()}));
            *ms += 16;
            Ok(())
        };
        for _ in 0..4 {
            motion::step(Duration::from_millis(16));
            presented_frame(a, cx).await;
            shot(a, &mut frames, &mut ms)?;
        }
        for s in &steps {
            inputs.push(ms);
            step_unsettled(a, &sp(s, "method"), s, cx)?;
            // Let the app take the input in (it draws its first frame of the change).
            cx.background_executor().timer(Duration::from_millis(40)).await;
            next_frame(a, cx).await;
            let mut quiet = 0;
            for _ in 0..60 {
                motion::step(Duration::from_millis(16));
                motion::take_animating();
                presented_frame(a, cx).await;
                shot(a, &mut frames, &mut ms)?;
                if motion::take_animating() {
                    quiet = 0;
                } else {
                    quiet += 1;
                    if quiet >= 3 {
                        break;
                    }
                }
            }
        }
        for _ in 0..4 {
            motion::step(Duration::from_millis(16));
            presented_frame(a, cx).await;
            shot(a, &mut frames, &mut ms)?;
        }
        Ok(())
    }
    .await;
    motion::release();
    settle(a, cx).await;
    result?;
    Ok(json!({"frames": frames, "inputs": inputs, "held": false}))
}

#[allow(dead_code)]
fn _unused(_: Result<()>) {}
