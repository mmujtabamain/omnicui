//! Settings (SPEC-PRODUCT.md, "taskscape.settings"): General, Capture, Shortcuts, About. In the
//! Tauri app it is a window built fresh on every open; here it is drawn in the main window, over
//! it, ready on its first frame (WHY.md). Every change applies at once and is saved.

use std::collections::BTreeMap;
use std::rc::Rc;

use gpui::{
    deferred, div, point, prelude::*, px, AnyElement, App, BoxShadow, Context, Entity, FocusHandle, Focusable, FontWeight, KeyDownEvent,
    MouseButton, Subscription, Window,
};
use serde_json::json;
use taskscape_common::hotkeys::{self, Accel, Key, Mods, ResolvedBinding, Scope};

use crate::db::Db;
use crate::keys::{Dismiss, PaletteDown, PaletteUp};
use crate::motion::{Anims, PANEL_IN};
use crate::parts::{self, part};
use crate::report;
use crate::text_field::{FieldEvent, TextField};
use crate::theme::{Theme, R_CONTROL, R_FIELD, R_PANEL};
use crate::ui::{icon, pressable, section_label, Handler};
use crate::workspace::Workspace;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pane {
    General,
    Capture,
    Shortcuts,
    About,
}
const PANES: [(Pane, &str, &str, &str); 4] = [
    (Pane::General, "navGeneral", "General", "tune"),
    (Pane::Capture, "navCapture", "Capture", "screenshot_monitor"),
    (Pane::Shortcuts, "navShortcuts", "Shortcuts", "keyboard"),
    (Pane::About, "navAbout", "About", "info"),
];

/// A row of a group: what search looks in.
struct RowSpec {
    id: &'static str,
    title: String,
    description: String,
    keywords: &'static str,
}

struct Group {
    label: String,
    rows: Vec<RowSpec>,
}

pub struct SettingsView {
    db: Rc<Db>,
    ws: Option<Entity<Workspace>>,
    system_dark: bool,
    pub focus: FocusHandle,
    pane: Pane,
    search: Entity<TextField>,
    opened: BTreeMap<String, String>,
    values: BTreeMap<String, String>,
    bindings: Vec<ResolvedBinding>,
    opened_bindings: Vec<ResolvedBinding>,
    recording: Option<&'static str>,
    error: Option<(&'static str, String)>,
    focus_by_part: BTreeMap<String, FocusHandle>,
    anims: Anims,
    serial: u64,
    _subs: Vec<Subscription>,
}

const KEYS: [&str; 4] = ["theme", "reduced_motion", "show_completed", "screenshot_mode"];

impl SettingsView {
    pub fn new(db: Rc<Db>, ws: Option<Entity<Workspace>>, system_dark: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextField::new("Search", window, cx));
        let subs = vec![cx.subscribe_in(&search, window, |this: &mut Self, _, e, window, cx| match e {
            FieldEvent::Cancel => {
                if this.search.read(cx).text().is_empty() {
                    this.close(window, cx)
                } else {
                    this.search.update(cx, |f, cx| f.set_text("", cx));
                }
            }
            _ => cx.notify(),
        })];
        let mut values = BTreeMap::new();
        for k in KEYS {
            let default = match k {
                "theme" => "system",
                "reduced_motion" => "0",
                "show_completed" => "1",
                _ => "fullscreen",
            };
            values.insert(k.to_string(), db.setting(k).unwrap_or_else(|| default.into()));
        }
        let bindings = db.hotkeys();
        SettingsView {
            db,
            ws,
            system_dark,
            focus: cx.focus_handle(),
            pane: Pane::General,
            search,
            opened: values.clone(),
            values,
            opened_bindings: bindings.clone(),
            bindings,
            recording: None,
            error: None,
            focus_by_part: BTreeMap::new(),
            anims: Anims::default(),
            serial: 0,
            _subs: subs,
        }
    }

    fn theme(&self) -> Theme {
        let dark = match self.values.get("theme").map(String::as_str) {
            Some("dark") => true,
            Some("light") => false,
            _ => self.system_dark,
        };
        Theme::of(dark)
    }

    fn fh(&mut self, name: &str, cx: &mut Context<Self>) -> FocusHandle {
        self.focus_by_part.entry(name.to_string()).or_insert_with(|| cx.focus_handle()).clone()
    }

    fn on(&self, cx: &Context<Self>, f: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static) -> Handler {
        let this = cx.entity().downgrade();
        Rc::new(move |window, cx| {
            this.update(cx, |s, cx| {
                f(s, window, cx);
                cx.notify();
            })
            .ok();
        })
    }

    fn set(&mut self, key: &str, value: &str, cx: &mut Context<Self>) {
        if self.values.get(key).map(String::as_str) == Some(value) {
            return;
        }
        self.db.set_setting(key, value);
        self.values.insert(key.to_string(), value.to_string());
        report(cx, "settingChanged", json!({ "key": key, "value": value }));
        if key == "reduced_motion" {
            crate::motion::set_reduced(value == "1");
        }
        if let Some(ws) = &self.ws {
            let (k, v) = (key.to_string(), value.to_string());
            ws.update(cx, |ws, cx| {
                match k.as_str() {
                    "show_completed" => ws.show_completed = v != "0",
                    "reduced_motion" => ws.reduced_motion = v == "1",
                    "theme" => ws.theme_setting = v.clone(),
                    "screenshot_mode" => ws.screenshot_mode = v.clone(),
                    _ => {}
                }
                cx.notify()
            });
        }
        cx.notify();
    }

    fn changed(&self) -> bool {
        self.values != self.opened || self.bindings.iter().zip(&self.opened_bindings).any(|(a, b)| a.accel != b.accel)
    }

    /// Puts back what Settings opened with.
    fn discard(&mut self, cx: &mut Context<Self>) {
        for (k, v) in self.opened.clone() {
            self.set(&k, &v, cx);
        }
        for b in self.opened_bindings.clone() {
            let cur = self.bindings.iter().find(|x| x.id == b.id).and_then(|x| x.accel);
            if cur != b.accel {
                let accel = b.accel.map(|a| a.to_string()).unwrap_or_default();
                if self.db.run(hotkeys::set_binding(&self.db.store, b.id, &accel)).is_ok() {
                    report(cx, "hotkeyChanged", json!({ "command": b.id, "accel": accel }));
                }
            }
        }
        self.reload_bindings(cx);
    }

    fn reload_bindings(&mut self, cx: &mut Context<Self>) {
        self.bindings = self.db.hotkeys();
        crate::keys::rebind(cx, &self.bindings);
        if let Some(ws) = &self.ws {
            let b = self.bindings.clone();
            ws.update(cx, |ws, _| ws.hotkeys = b);
        }
        cx.notify();
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        report(cx, "windowAction", json!({ "action": "close" }));
        if let Some(ws) = self.ws.clone() {
            ws.update(cx, |ws, cx| ws.close_settings(window, cx));
        }
    }

    fn groups(&self, pane: Pane) -> Vec<Group> {
        let r = |id: &'static str, title: &str, description: &str, keywords: &'static str| RowSpec { id, title: title.into(), description: description.into(), keywords };
        match pane {
            Pane::General => vec![
                Group {
                    label: "Appearance".into(),
                    rows: vec![
                        r("theme", "Theme", "Follow the system appearance, or pin one.", "dark light mode colour color appearance system"),
                        r("reduced_motion", "Reduced motion", "Cuts animations and transitions to the minimum.", "animation animations transitions battery low power motion"),
                    ],
                },
                Group { label: "Tasks".into(), rows: vec![r("show_completed", "Show completed tasks", "Keep finished tasks in the list instead of hiding them.", "done finished hide completed checked")] },
            ],
            Pane::Capture => vec![Group {
                label: "Screenshots".into(),
                rows: vec![
                    r("screenshot_mode", "Capture mode", "", "screenshot capture region area window full screen"),
                    r("saved_to", "Saved to", "Click to open the folder.", "folder path directory location disk screenshots finder"),
                ],
            }],
            Pane::Shortcuts => {
                let mut out = vec![];
                for (scope, label) in [(Scope::Global, "Global"), (Scope::Main, "Main window"), (Scope::Tray, "Capture bar")] {
                    let rows = self.bindings.iter().filter(|b| b.scope == scope).map(|b| RowSpec { id: b.id, title: b.label.into(), description: String::new(), keywords: "shortcut hotkey key" }).collect();
                    out.push(Group { label: label.into(), rows });
                }
                out
            }
            Pane::About => vec![
                Group {
                    label: "Application".into(),
                    rows: vec![
                        r("version", "Taskscape", "A macOS task manager with a menu-bar capture bar.", "version about"),
                        r("data_dir", "Data folder", "Tasks, attachments, screenshots and preferences. Click to open it.", "path directory location database sqlite backup storage finder"),
                    ],
                },
                Group {
                    label: "Reset".into(),
                    rows: vec![r(
                        "reset_all",
                        "Restore defaults",
                        "Puts appearance, capture and task preferences — and every keyboard shortcut — back the way they shipped. Your tasks are untouched.",
                        "reset clear defaults factory revert erase preferences",
                    )],
                },
            ],
        }
    }

    fn filtered(&self, pane: Pane, q: &str) -> Vec<Group> {
        let tokens: Vec<String> = q.to_lowercase().split_whitespace().map(String::from).collect();
        let mut gs = self.groups(pane);
        if tokens.is_empty() {
            return gs;
        }
        for g in &mut gs {
            let label = g.label.clone();
            g.rows.retain(|r| {
                let hay = format!("{} {} {} {}", label, r.title, r.description, r.keywords).to_lowercase();
                tokens.iter().all(|t| hay.contains(t))
            });
        }
        gs.retain(|g| !g.rows.is_empty());
        gs
    }

    fn on_record_key(&mut self, e: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.recording else { return };
        cx.stop_propagation();
        let k = &e.keystroke;
        let m = &k.modifiers;
        if k.key == "escape" {
            self.recording = None;
            cx.notify();
            return;
        }
        let bare = !m.platform && !m.control && !m.alt && !m.shift;
        let accel = if bare && (k.key == "backspace" || k.key == "delete") {
            Some(String::new())
        } else {
            key_from_gpui(&k.key).map(|key| {
                let mut mods = Mods::NONE;
                for (on, b) in [(m.platform, Mods::CMD), (m.control, Mods::CTRL), (m.alt, Mods::ALT), (m.shift, Mods::SHIFT)] {
                    if on {
                        mods = mods.with(b);
                    }
                }
                Accel::new(mods, key).to_string()
            })
        };
        let Some(accel) = accel else { return };
        self.recording = None;
        match self.db.run(hotkeys::set_binding(&self.db.store, id, &accel)) {
            Ok(()) => {
                self.error = None;
                report(cx, "hotkeyChanged", json!({ "command": id, "accel": accel }));
                self.reload_bindings(cx);
            }
            Err(err) => self.error = Some((id, format!("{err}"))),
        }
        cx.notify();
    }

    fn nav_step(&mut self, by: isize, cx: &mut Context<Self>) {
        let i = PANES.iter().position(|p| p.0 == self.pane).unwrap_or(0) as isize;
        let n = (i + by).clamp(0, PANES.len() as isize - 1) as usize;
        self.pane = PANES[n].0;
        cx.notify();
    }

    fn render_row(&mut self, row: &RowSpec, t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let control: AnyElement = match row.id {
            "theme" => self.segmented("theme", &[("themeSystem", "System", "system", "contrast"), ("themeLight", "Light", "light", "light_mode"), ("themeDark", "Dark", "dark", "dark_mode")], t, window, cx),
            "screenshot_mode" => self.segmented("screenshot_mode", &[("captureModeFullscreen", "Full screen", "fullscreen", "open_in_full"), ("captureModeRegion", "Region", "region", "crop_square")], t, window, cx),
            "reduced_motion" | "show_completed" => {
                let on = self.values.get(row.id).map(String::as_str) == Some("1");
                let name = if row.id == "reduced_motion" { "reducedMotion" } else { "showCompleted" };
                let fh = self.fh(name, cx);
                let key = row.id;
                // Toggles never animate (DESIGN.md, Motion).
                part(name, "switch")
                    .checked(on)
                    .name(row.title.clone())
                    .focus(&fh, window)
                    .wrap(
                        pressable(name, &fh, self.on(cx, move |s, _, cx| s.set(key, if on { "0" } else { "1" }, cx)))
                            .w(px(30.))
                            .h(px(18.))
                            .flex_none()
                            .rounded_full()
                            .p(px(2.))
                            .flex()
                            .when(on, |d| d.bg(t.accent_500).justify_end())
                            .when(!on, |d| d.bg(t.surface_0).border_1().border_color(t.edge_2).justify_start())
                            .when(fh.is_focused(window), |d| d.border_2().border_color(t.focus_1))
                            .child(div().size(px(14.)).rounded_full().bg(t.surface_3).shadow_sm()),
                    )
                    .into_any_element()
            }
            "saved_to" | "data_dir" => {
                let path = if row.id == "saved_to" { taskscape_common::paths::screenshots_dir() } else { taskscape_common::paths::root_dir() };
                let p = path.display().to_string();
                let fh = self.fh(row.id, cx);
                let p2 = p.clone();
                pressable(row.id, &fh, self.on(cx, move |_, _, cx| cx.open_url(&format!("file://{p2}"))))
                    .h(px(26.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .rounded(px(R_CONTROL))
                    .bg(t.surface_0)
                    .text_size(px(12.))
                    .text_color(t.content_2)
                    .child(icon("folder_open", 14., t.content_3))
                    .child(div().max_w(px(240.)).truncate().child(p))
                    .into_any_element()
            }
            "version" => div().text_size(px(12.)).text_color(t.content_3).child(env!("CARGO_PKG_VERSION")).into_any_element(),
            "reset_all" => {
                let fh = self.fh("resetAll", cx);
                pressable("resetAll", &fh, self.on(cx, |s, _, cx| {
                    for (k, v) in [("theme", "system"), ("reduced_motion", "0"), ("show_completed", "1"), ("screenshot_mode", "fullscreen")] {
                        s.set(k, v, cx);
                    }
                    for b in s.bindings.clone() {
                        let _ = s.db.run(hotkeys::reset_binding(&s.db.store, b.id));
                    }
                    s.reload_bindings(cx);
                }))
                .h(px(28.))
                .px(px(10.))
                .flex()
                .items_center()
                .rounded(px(R_CONTROL))
                .border_1()
                .border_color(t.edge_2)
                .text_size(px(13.))
                .text_color(t.danger_500)
                .child("Restore defaults")
                .into_any_element()
            }
            id => self.keycap(id, t, window, cx),
        };
        let stacked = matches!(row.id, "theme" | "screenshot_mode" | "saved_to" | "data_dir" | "reset_all");
        let mut text = div().flex().flex_col().gap(px(2.)).child(div().text_size(px(13.5)).font_weight(FontWeight::MEDIUM).child(row.title.clone()));
        if !row.description.is_empty() {
            text = text.child(div().text_size(px(12.)).line_height(px(16.)).text_color(t.content_3).child(row.description.clone()));
        }
        let mut r = div().px(px(9.)).py(px(8.)).border_b_1().border_color(t.edge_1);
        r = if stacked { r.flex().flex_col().gap(px(6.)).child(text).child(control) } else { r.flex().flex_row().items_center().gap(px(12.)).child(div().flex_1().child(text)).child(control) };
        if let Some((id, msg)) = self.error.as_ref().filter(|(id, _)| *id == row.id) {
            let _ = id;
            r = r.child(part("hotkeyError", "status").text(msg.clone()).wrap(div().text_size(px(11.5)).text_color(t.danger_500).child(msg.clone())));
        }
        r.into_any_element()
    }

    fn segmented(&mut self, key: &'static str, opts: &[(&'static str, &'static str, &'static str, &'static str)], t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let cur = self.values.get(key).cloned().unwrap_or_default();
        let mut row = div().flex().flex_row().p(px(2.)).gap(px(2.)).rounded(px(R_CONTROL)).bg(t.surface_0);
        for (name, label, value, ic) in opts.iter().copied() {
            let fh = self.fh(name, cx);
            let on = cur == value;
            row = row.child(
                part(name, "button").text(label).selected(on).focus(&fh, window).wrap(
                    pressable(name, &fh, self.on(cx, move |s, _, cx| s.set(key, value, cx)))
                        .flex_1()
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap(px(4.))
                        .rounded(px(R_FIELD))
                        .text_size(px(12.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .when(on, |d| d.bg(t.surface_3).text_color(t.content_1).shadow_sm())
                        .when(!on, |d| d.text_color(t.content_3).hover(|s| s.bg(t.wash_2)))
                        .when(fh.is_focused(window), |d| d.border_2().border_color(t.focus_1))
                        .child(icon(ic, 13., if on { t.content_1 } else { t.content_3 }))
                        .child(label),
                ),
            );
        }
        row.into_any_element()
    }

    fn keycap(&mut self, id: &str, t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(b) = self.bindings.iter().find(|b| b.id == id).cloned() else { return div().into_any_element() };
        let name = format!("hotkey{}", camel(id));
        let fh = self.fh(&name, cx);
        let recording = self.recording == Some(b.id);
        let text = if recording { "Type shortcut…".to_string() } else { b.accel.map(|a| a.glyphs()).unwrap_or_else(|| "—".into()) };
        let bid = b.id;
        let changed = b.accel != Some(b.default);
        let mut row = div().flex().flex_row().items_center().gap(px(4.));
        if changed {
            let fh_r = self.fh(&format!("{name}Reset"), cx);
            row = row.child(
                pressable(gpui::SharedString::from(format!("{name}Reset")), &fh_r, self.on(cx, move |s, _, cx| {
                    if s.db.run(hotkeys::reset_binding(&s.db.store, bid)).is_ok() {
                        let accel = s.db.hotkeys().into_iter().find(|x| x.id == bid).and_then(|x| x.accel).map(|a| a.to_string()).unwrap_or_default();
                        report(cx, "hotkeyChanged", json!({ "command": bid, "accel": accel }));
                    }
                    s.reload_bindings(cx);
                }))
                .size(px(22.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(R_FIELD))
                .hover(|s| s.bg(t.wash_2))
                .child(icon("restart_alt", 14., t.content_3)),
            );
        }
        row = row.child(
            part(name.clone(), "button").text(text.clone()).name(b.label).focus(&fh, window).wrap(
                pressable(gpui::SharedString::from(name), &fh, self.on(cx, move |s, w, cx| {
                    s.error = None;
                    s.recording = if s.recording == Some(bid) { None } else { Some(bid) };
                    let f = s.focus_by_part.get(&format!("hotkey{}", camel(bid))).cloned();
                    if let Some(f) = f {
                        w.focus(&f);
                    }
                    cx.notify()
                }))
                .on_key_down(cx.listener(Self::on_record_key))
                .h(px(24.))
                .min_w(px(46.))
                .px(px(8.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(R_FIELD))
                .bg(if recording { t.selection_1 } else { t.surface_0 })
                .border_1()
                .border_color(if recording { t.accent_500 } else { t.edge_2 })
                .text_size(px(12.5))
                .font_weight(FontWeight::MEDIUM)
                .text_color(if recording { t.accent_500 } else { t.content_1 })
                .child(text),
            ),
        );
        row.into_any_element()
    }

    pub fn render_content(&mut self, titlebar: bool, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let q = self.search.read(cx).text().to_string();
        let searching = !q.trim().is_empty();
        self.search.update(cx, |f, _| {
            f.caret = t.accent_500;
            f.placeholder_color = t.content_3;
            f.selection_color = t.selection_1;
        });
        let sfh = self.search.read(cx).focus.clone();

        // The sidebar.
        let mut side = div().w(px(154.)).flex_none().h_full().flex().flex_col().gap(px(2.)).p(px(7.)).bg(t.surface_1).border_r_1().border_color(t.edge_2);
        let mut well = div()
            .h(px(28.))
            .mb(px(6.))
            .px(px(8.))
            .flex()
            .items_center()
            .gap(px(6.))
            .rounded(px(R_CONTROL))
            .bg(t.surface_0)
            .text_size(px(12.5))
            .child(icon("search", 13., t.content_3))
            .child(part("settingsSearch", "textbox").text(q.clone()).name("Search settings").focus(&sfh, window).wrap(div().flex_1().min_w_0().child(self.search.clone())));
        if searching {
            let fh = self.fh("settingsClearSearch", cx);
            well = well.child(
                part("settingsClearSearch", "button").name("Clear search").focus(&fh, window).wrap(
                    pressable("settingsClearSearch", &fh, self.on(cx, |s, w, cx| {
                        s.search.update(cx, |f, cx| f.set_text("", cx));
                        let f = s.search.read(cx).focus.clone();
                        w.focus(&f);
                    }))
                    .size(px(18.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon("close", 13., t.content_3)),
                ),
            );
        }
        side = side.child(well);
        for (pane, name, label, ic) in PANES {
            let fh = self.fh(name, cx);
            let count = if searching { Some(self.filtered(pane, &q).iter().map(|g| g.rows.len()).sum::<usize>()) } else { None };
            let on = self.pane == pane;
            let text = match count {
                Some(n) if n > 0 => format!("{label} {n}"),
                _ => label.to_string(),
            };
            side = side.child(
                part(name, "button").text(text).name(label).selected(on).focus(&fh, window).wrap(
                    pressable(name, &fh, self.on(cx, move |s, _, cx| {
                        s.pane = pane;
                        s.serial += 1;
                        cx.notify()
                    }))
                    .h(px(30.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .rounded(px(R_CONTROL))
                    .text_size(px(13.5))
                    .when(on, |d| d.bg(t.selection_1))
                    .when(!on, |d| d.hover(|s| s.bg(t.wash_1)))
                    .when(fh.is_focused(window), |d| d.border_2().border_color(t.focus_1))
                    .when(searching && count == Some(0), |d| d.opacity(0.5))
                    .child(icon(ic, 15., t.content_3))
                    .child(div().flex_1().child(label))
                    .children(count.filter(|n| *n > 0).map(|n| div().text_size(px(11.)).font_weight(FontWeight::SEMIBOLD).text_color(t.accent_500).child(n.to_string()))),
                ),
            );
        }

        // The pane.
        let p = self.anims.progress("pane", format!("{:?}{}", self.pane, self.serial), PANEL_IN, window);
        let mut body = div().id("settingsBody").flex_1().min_w_0().h_full().overflow_y_scroll().bg(t.surface_2).p(px(17.)).flex().flex_col().gap(px(14.));
        let panes: Vec<(Pane, &str)> = if searching { PANES.iter().map(|p| (p.0, p.2)).collect() } else { vec![(self.pane, "")] };
        let mut any = false;
        let narrowed = searching && PANES.iter().any(|x| x.0 == self.pane) && !self.filtered(self.pane, &q).is_empty();
        for (pane, plabel) in panes {
            if narrowed && pane != self.pane {
                continue;
            }
            let groups = self.filtered(pane, &q);
            if pane == Pane::Shortcuts && !searching {
                let intro = "Click a shortcut to change it — press the new keys, Backspace to remove, Esc to cancel.";
                body = body.child(part("settingsIntro", "generic").text(intro).wrap(div().text_size(px(12.5)).line_height(px(17.)).text_color(t.content_3).child(intro)));
            }
            for g in groups {
                any = true;
                let label = if searching { format!("{plabel} · {}", g.label) } else { g.label.clone() };
                let mut card = div().rounded(px(R_PANEL)).border_1().border_color(t.edge_2).bg(t.surface_3).overflow_hidden();
                for row in &g.rows {
                    card = card.child(self.render_row(row, &t, window, cx));
                }
                body = body.child(div().flex().flex_col().gap(px(6.)).child(div().flex().items_center().gap(px(8.)).child(section_label(&label, &t)).child(div().flex_1().h(px(1.)).bg(t.edge_1))).child(card));
            }
        }
        if !any {
            let sub = format!("Nothing here for “{}”.", q.trim());
            body = body.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(6.))
                    .pt(px(120.))
                    .child(icon("search", 28., t.content_3))
                    .child(part("settingsEmptyTitle", "heading").text("No settings match").wrap(div().text_size(px(15.)).font_weight(FontWeight::MEDIUM).text_color(t.content_2).child("No settings match")))
                    .child(part("settingsEmptySubtitle", "generic").text(sub.clone()).wrap(div().text_size(px(12.5)).text_color(t.content_3).child(sub))),
            );
        }
        let body = div().flex_1().h_full().relative().left(px(6. * (1. - p))).opacity(p).child(body);

        let mut col = div().size_full().flex().flex_col();
        if titlebar {
            let mut tb = div().h(px(46.)).flex_none().flex().flex_row().items_center().px(px(16.)).bg(t.surface_1).border_b_1().border_color(t.edge_2).child(div().flex_1().text_size(px(14.)).font_weight(FontWeight::SEMIBOLD).child("Settings"));
            if self.changed() {
                let fh = self.fh("discardChanges", cx);
                tb = tb.child(
                    part("discardChanges", "button").name("Discard changes").focus(&fh, window).wrap(
                        pressable("discardChanges", &fh, self.on(cx, |s, _, cx| s.discard(cx)))
                            .h(px(26.))
                            .px(px(8.))
                            .flex()
                            .items_center()
                            .gap(px(4.))
                            .rounded(px(R_CONTROL))
                            .text_size(px(12.5))
                            .text_color(t.content_2)
                            .hover(|s| s.bg(t.wash_2))
                            .child(icon("undo", 14., t.content_3))
                            .child("Discard changes"),
                    ),
                );
            }
            if self.ws.is_some() {
                let fh = self.fh("settingsClose", cx);
                tb = tb.child(
                    pressable("settingsClose", &fh, self.on(cx, |s, w, cx| s.close(w, cx)))
                        .ml(px(6.))
                        .size(px(26.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(R_CONTROL))
                        .hover(|s| s.bg(t.wash_2))
                        .child(icon("close", 15., t.content_3)),
                );
            }
            col = col.child(part("settingsTitlebar", "generic").text("Settings").wrap(tb));
        }
        col.child(div().flex_1().min_h_0().flex().flex_row().child(side).child(body)).into_any_element()
    }
}

fn camel(id: &str) -> String {
    id.split('_').map(|w| { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() }).collect()
}

fn key_from_gpui(k: &str) -> Option<Key> {
    let token = match k {
        "enter" => "Enter",
        "backspace" => "Backspace",
        "delete" => "Delete",
        "space" => "Space",
        "tab" => "Tab",
        "escape" => "Escape",
        "up" => "ArrowUp",
        "down" => "ArrowDown",
        "left" => "ArrowLeft",
        "right" => "ArrowRight",
        "shift" | "control" | "alt" | "platform" | "function" => return None,
        other => other,
    };
    Key::parse(token)
}

impl Focusable for SettingsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();
        let content = self.render_content(true, window, cx);
        let root = div()
            .id("settings")
            .key_context("Settings")
            .track_focus(&self.focus)
            .size_full()
            .font_family(crate::system_font())
            .text_size(px(14.))
            .line_height(px(19.))
            .text_color(t.content_1)
            .bg(t.surface_2)
            .on_action(cx.listener(|s, _: &PaletteUp, _, cx| s.nav_step(-1, cx)))
            .on_action(cx.listener(|s, _: &PaletteDown, _, cx| s.nav_step(1, cx)))
            .on_action(cx.listener(|s, _: &Dismiss, w, cx| {
                if s.recording.is_some() {
                    return;
                }
                if !s.search.read(cx).text().is_empty() {
                    s.search.update(cx, |f, cx| f.set_text("", cx));
                } else {
                    s.close(w, cx)
                }
            }))
            .child(content);
        if self.ws.is_some() {
            root.into_any_element()
        } else {
            parts::frame(root).into_any_element()
        }
    }
}

/// Settings drawn over the main window: a 640×520 panel that scales in.
pub fn modal(view: Entity<SettingsView>, t: &Theme, window: &mut Window, cx: &mut Context<Workspace>) -> AnyElement {
    let _ = window;
    let scrim = gpui::hsla(0., 0., 0., 0.30);
    let card = div()
        .w(px(640.))
        .h(px(520.))
        .rounded(px(R_PANEL))
        .overflow_hidden()
        .border_1()
        .border_color(t.edge_2)
        .shadow(vec![
            BoxShadow { color: t.shadow_alpha(0.20), offset: point(px(0.), px(10.)), blur_radius: px(28.), spread_radius: px(0.) },
            BoxShadow { color: t.shadow_alpha(0.12), offset: point(px(0.), px(1.)), blur_radius: px(3.), spread_radius: px(0.) },
        ])
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(view);
    let _ = cx;
    deferred(div().id("settingsScrim").absolute().inset_0().size_full().bg(scrim).occlude().flex().items_center().justify_center().child(card)).with_priority(4).into_any_element()
}
