//! omnicui's egui target: the adapter. omnicui starts it once per check (`adapter:` in target.yaml), writes
//! one JSON-RPC 2.0 request per line on standard input, and reads exactly one reply per request,
//! one line each, from standard output. Logs go to standard error only.
//!
//! Components run inside egui_kittest's headless harness. Parts are found in the AccessKit tree
//! (the tree a screen reader reads) by the `author_id` each component sets on them; clicks and keys
//! are real egui input events; screenshots are rendered with wgpu. Time is simulated: every frame
//! advances by the same step, so a traced animation gives the same samples on every run.
use egui::{Key, Modifiers, Pos2, Rect, UiBuilder, ViewportId};
use egui_kittest::kittest::NodeT;
use egui_kittest::{Harness, Node};
mod components;

use components::{create, Mounted};
use desktop::{fonts, Look};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::time::Duration;

// protocol: error codes
const NO_COMPONENT: i64 = -32001;
const NO_PART: i64 = -32002;
const STEP_FAILED: i64 = -32003;
const NO_METHOD: i64 = -32601;

/// Simulated time per frame: 60 frames a second.
const STEP_DT: f32 = 1.0 / 60.0;

struct Rpc(i64, String);

type Result<T> = std::result::Result<T, Rpc>;

fn fail(code: i64, msg: impl Into<String>) -> Rpc {
    Rpc(code, msg.into())
}

struct AppState {
    component: Box<dyn Mounted>,
    events: Vec<Value>,
    look: Look,
}

struct Target {
    harness: Option<Harness<'static, AppState>>,
    can_render: Option<String>,
}

fn key_of(name: &str) -> Result<Key> {
    Key::from_name(name).ok_or_else(|| fail(STEP_FAILED, format!("unknown key {name}")))
}

/// "Shift+Tab" → (Shift, Tab)
fn keys(combo: &str) -> Result<(Modifiers, Key)> {
    let mut mods = Modifiers::NONE;
    let mut parts: Vec<&str> = combo.split('+').collect();
    let key = key_of(parts.pop().unwrap_or_default())?;
    for m in parts {
        match m {
            "Shift" => mods.shift = true,
            "Control" => mods.ctrl = true,
            "Alt" => mods.alt = true,
            "Meta" => mods.command = true,
            other => return Err(fail(STEP_FAILED, format!("unknown modifier {other}"))),
        }
    }
    Ok((mods, key))
}

fn build(component: Box<dyn Mounted>, look: Look, size: egui::Vec2, dark: bool) -> Harness<'static, AppState> {
    Harness::builder()
        .with_size(size)
        .with_pixels_per_point(1.0)
        .with_step_dt(STEP_DT)
        .with_max_steps(600)
        // egui_kittest can refuse unnamed widgets itself, by panicking. The contract's tests are
        // the judge here, so that a missing name is reported like on every other target.
        .with_accessibility_check(false)
        // Screenshots show the component, not the simulated mouse pointer.
        .with_render_cursor(target_defect("egui.render.cursor"))
        .with_theme(if dark { egui::Theme::Dark } else { egui::Theme::Light })
        .with_context_setup(move |ctx| {
            fonts(ctx);
            ctx.all_styles_mut(|s| {
                for w in [&mut s.visuals.widgets.inactive, &mut s.visuals.widgets.hovered, &mut s.visuals.widgets.active] {
                    w.expansion = 0.0;
                }
            });
        })
        .build_ui_state(
            |ui, s: &mut AppState| {
                let screen = ui.ctx().content_rect();
                ui.painter().rect_filled(screen, 0.0, s.look.bg);
                // Every target puts the component at (24, 24).
                let inner = Rect::from_min_max(Pos2::new(24.0, 24.0), screen.max - egui::vec2(24.0, 24.0));
                let AppState { component, events, look } = s;
                ui.scope_builder(UiBuilder::new().max_rect(inner), |ui| {
                    component.frame(ui, *look, &mut |name, fields| events.push(json!({ "name": name, "fields": fields })));
                });
            },
            AppState { component, events: vec![], look },
        )
}

impl Target {
    fn harness(&mut self) -> Result<&mut Harness<'static, AppState>> {
        self.harness.as_mut().ok_or_else(|| fail(STEP_FAILED, "nothing is mounted"))
    }

    /// Runs frames until nothing is animating and no repaint is asked for: the UI has settled.
    fn settle(&mut self) -> Result<()> {
        let h = self.harness()?;
        h.try_run().map(|_| ()).map_err(|e| fail(STEP_FAILED, format!("the UI did not settle: {e}")))
    }

    fn handle(&mut self, method: &str, p: &Value) -> Result<Value> {
        let part = p["part"].as_str().unwrap_or_default().to_owned();
        match method {
            // protocol: the first request; the constructs this target can perform on this run.
            "advertise" => {
                let mut can = vec!["mount", "press", "focus", "key", "type", "drag", "hover", "read", "events", "trace"];
                let mut cannot = serde_json::Map::new();
                match &self.can_render {
                    None => can.extend(["screenshot", "record"]),
                    Some(why) => {
                        cannot.insert("screenshot".into(), json!(why));
                        cannot.insert("record".into(), json!(why));
                    }
                }
                Ok(json!({ "protocol": "1", "can": can, "cannot": cannot }))
            }
            // contract: tests[].props, viewports, colorSchemes; a fresh harness for every test.
            "mount" => {
                let component = p["component"].as_str().unwrap_or_default();
                let c = create(component, p["props"].clone()).map_err(|e| fail(if e.starts_with("no component") { NO_COMPONENT } else { STEP_FAILED }, e))?;
                let scheme = p["colorScheme"].as_str().unwrap_or("light");
                let size = egui::vec2(p["viewport"]["width"].as_f64().unwrap_or(320.0) as f32, p["viewport"]["height"].as_f64().unwrap_or(240.0) as f32);
                self.harness = Some(build(c, Look::of(scheme), size, scheme == "dark"));
                self.settle()?;
                Ok(Value::Null)
            }
            // contract: tests[].steps[].action on parts.<name>, sent as the part id "ui.counter.<name>".
            "press" | "focus" | "key" | "type" | "drag" | "hover" => {
                self.queue_step(method, p)?;
                self.settle()?;
                Ok(Value::Null)
            }
            // contract: tests[].expect[] with part:. Everything is read from AccessKit.
            "read" => {
                let h = self.harness()?;
                let root = h.root();
                let node = find(&root, &part)?;
                let n = node.accesskit_node();
                Ok(match p["field"].as_str().unwrap_or_default() {
                    "text" => json!(n.value().or_else(|| n.label())),
                    "name" => json!(n.label()),
                    "checked" => json!(n.toggled().map(|t| t == egui::accesskit::Toggled::True)),
                    "disabled" => json!(n.is_disabled()),
                    "value" => json!(n.numeric_value()),
                    other => return Err(fail(STEP_FAILED, format!("unknown field {other}"))),
                })
            }
            // contract: tests[].expect[] with event:
            "events" => {
                let h = self.harness()?;
                Ok(Value::Array(std::mem::take(&mut h.state_mut().events)))
            }
            // contract: tests[].screenshot: true; the path is chosen by omnicui.
            "screenshot" => {
                self.settle()?;
                let path = p["path"].as_str().unwrap_or_default().to_owned();
                self.render_to(&path)?;
                Ok(json!(path))
            }
            // contract: tests[].trace[]: perform the steps frame by frame, reading the part's box.
            "trace" => {
                let point = p["point"].as_str().unwrap_or("center").to_owned();
                let samples = self.frames(p, |t, frame_ms| {
                    let h = t.harness()?;
                    let root = h.root();
                    let r = find(&root, &part)?.rect();
                    let at = if point == "center" { r.center() } else { r.min };
                    Ok(json!({ "ms": frame_ms, "x": at.x, "y": at.y }))
                })?;
                Ok(Value::Array(samples))
            }
            // contract: tests[].record: true; a frame of the clip for every simulated frame.
            "record" => {
                let dir = p["path"].as_str().unwrap_or_default().to_owned();
                std::fs::create_dir_all(&dir).map_err(|e| fail(STEP_FAILED, e.to_string()))?;
                let mut n = 0;
                let frames = self.frames(p, |t, frame_ms| {
                    let file = format!("{dir}/{n}.png");
                    n += 1;
                    t.render_to(&file)?;
                    Ok(json!({ "ms": frame_ms, "file": file }))
                })?;
                Ok(Value::Array(frames))
            }
            other => Err(fail(NO_METHOD, format!("unknown method {other}"))),
        }
    }

    /// Queues one step's input events; the next frames process them.
    fn queue_step(&mut self, method: &str, p: &Value) -> Result<()> {
        let part = p["part"].as_str().unwrap_or_default().to_owned();
        let h = self.harness()?;
        let root = h.root();
        let node = find(&root, &part)?;
        match method {
            "press" => node.click(),
            "hover" => node.hover(),
            "focus" => node.focus(),
            "type" => node.type_text(p["text"].as_str().unwrap_or_default()),
            // The key goes wherever egui's focus really is.
            "key" => {
                let (mods, key) = keys(p["key"].as_str().unwrap_or_default())?;
                h.key_press_modifiers(mods, key);
            }
            "drag" => {
                let from = node.rect().center();
                let to = from + egui::vec2(p["dx"].as_f64().unwrap_or(0.0) as f32, p["dy"].as_f64().unwrap_or(0.0) as f32);
                h.drag_at(from);
                h.hover_at(to);
                h.drop_at(to);
            }
            other => return Err(fail(NO_METHOD, format!("unknown step {other}"))),
        }
        Ok(())
    }

    /// Runs the given steps one simulated frame at a time, calling `sample` after every frame,
    /// until the UI has settled.
    fn frames(&mut self, p: &Value, mut sample: impl FnMut(&mut Self, f64) -> Result<Value>) -> Result<Vec<Value>> {
        let mut out = vec![];
        let mut frame = 0u32;
        let ms = |f: u32| f as f64 * STEP_DT as f64 * 1000.0;
        out.push(sample(self, ms(frame))?);
        let steps = p["steps"].as_array().cloned().unwrap_or_default();
        for s in &steps {
            self.queue_step(s["method"].as_str().unwrap_or_default(), s)?;
            let mut quiet = 0;
            while quiet < 3 && frame < 600 {
                let h = self.harness()?;
                h.step();
                frame += 1;
                let delay = h.output().viewport_output.get(&ViewportId::ROOT).map(|v| v.repaint_delay).unwrap_or(Duration::MAX);
                quiet = if delay == Duration::ZERO { 0 } else { quiet + 1 };
                out.push(sample(self, ms(frame))?);
            }
        }
        Ok(out)
    }

    fn render_to(&mut self, path: &str) -> Result<()> {
        if let Some(why) = &self.can_render {
            return Err(fail(STEP_FAILED, why.clone()));
        }
        let h = self.harness()?;
        let img = h.render().map_err(|e| fail(STEP_FAILED, format!("render: {e}")))?;
        if let Some(dir) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        img.save(path).map_err(|e| fail(STEP_FAILED, format!("write {path}: {e}")))
    }
}

/// The target's own deliberate defects (the harness's, not the app's), with OMNICUI_DEFECTS.
fn target_defect(name: &str) -> bool {
    std::env::var("OMNICUI_DEFECTS").unwrap_or_default().split(',').any(|d| d == name)
}

/// The interface with components: a part is the AccessKit node whose author_id is its part id.
fn find<'t>(root: &Node<'t>, part: &str) -> Result<Node<'t>> {
    if root.accesskit_node().author_id() == Some(part) {
        return Ok(*root);
    }
    root.children_recursive()
        .find(|n| n.accesskit_node().author_id() == Some(part))
        .ok_or_else(|| fail(NO_PART, format!("no part {part}")))
}

/// Can this machine render? A tiny harness is rendered once at start-up.
/// OMNICUI_EGUI_RENDER=off says no without trying, to test a machine that can't.
fn probe_render() -> Option<String> {
    if std::env::var("OMNICUI_EGUI_RENDER").as_deref() == Ok("off") {
        return Some("rendering switched off (OMNICUI_EGUI_RENDER=off)".into());
    }
    struct Empty;
    impl Mounted for Empty {
        fn frame(&mut self, ui: &mut egui::Ui, _: Look, _: &mut dyn FnMut(&'static str, Value)) {
            ui.label("probe");
        }
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut h = build(Box::new(Empty), Look::of("light"), egui::vec2(32.0, 32.0), false);
        h.render().map(|_| ())
    }));
    match result {
        Ok(Ok(())) => None,
        Ok(Err(e)) => Some(format!("no GPU adapter for wgpu ({})", e.lines().next().unwrap_or_default())),
        Err(_) => Some("no GPU adapter for wgpu".into()),
    }
}

fn main() {
    let can_render = probe_render();
    if let Some(why) = &can_render {
        eprintln!("[egui] screenshots off: {why}");
    }
    let mut target = Target { harness: None, can_render };
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Err(_) => json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": "parse error" } }),
            Ok(req) => {
                let method = req["method"].as_str().unwrap_or_default().to_owned();
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| target.handle(&method, &req["params"]))).unwrap_or_else(|p| {
                    let why = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    Err(fail(STEP_FAILED, format!("{method} panicked: {}", why.lines().next().unwrap_or_default())))
                });
                match result {
                    Ok(v) => json!({ "jsonrpc": "2.0", "id": req["id"], "result": v }),
                    Err(Rpc(code, message)) => {
                        eprintln!("[egui] {method} failed: {message}");
                        json!({ "jsonrpc": "2.0", "id": req["id"], "error": { "code": code, "message": message } })
                    }
                }
            }
        };
        // protocol: exactly one reply per request, one line, on standard output
        writeln!(out, "{reply}").and_then(|_| out.flush()).expect("write a reply");
    }
}
