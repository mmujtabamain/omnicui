//! The capture bar (SPEC-PRODUCT.md, "taskscape.capture"): the hero. A native pop-up window that
//! is ready when summoned (WHY.md), with one card: the title, the notes behind Tab, where the
//! task will go, and the screenshot button. The draft survives dismissal.

use std::rc::Rc;
use std::time::Duration;

use gpui::{div, prelude::*, px, App, Context, Entity, FocusHandle, Focusable, FontWeight, Subscription, Task as GpuiTask, Window};
use serde_json::json;
use taskscape_common::{storage::strip_html, List};

use crate::db::Db;
use crate::keys::{CaptureSubmit, ClearDraft, Dismiss};
use crate::motion::Anims;
use crate::parts::{self, part};
use crate::report;
use crate::text_field::{FieldEvent, TextField};
use crate::theme::Theme;
use crate::ui::{icon, pressable};

#[derive(Clone, PartialEq)]
enum Shot {
    Idle,
    Capturing,
    Failed,
}

pub struct CaptureBar {
    db: Rc<Db>,
    system_dark: bool,
    pub focus: FocusHandle,
    title: Entity<TextField>,
    notes: Entity<TextField>,
    notes_open: bool,
    /// Dismissed: the bar is hidden, with its draft kept for the next summons.
    hidden: bool,
    shots: Vec<String>,
    shot: Shot,
    anims: Anims,
    focus_by_part: std::collections::BTreeMap<String, FocusHandle>,
    _shot_timer: Option<GpuiTask<()>>,
    _subs: Vec<Subscription>,
    /// What the app does when the bar is dismissed or asks for the main window.
    pub on_dismiss: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    pub on_open_main: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl CaptureBar {
    pub fn new(db: Rc<Db>, system_dark: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let title = cx.new(|cx| TextField::new("Capture a task ...", window, cx));
        let notes = cx.new(|cx| {
            let mut f = TextField::new("Notes ...", window, cx);
            f.multiline = true;
            f
        });
        let subs = vec![
            cx.subscribe_in(&title, window, |this: &mut Self, _, e, window, cx| match e {
                FieldEvent::Submit => this.submit(window, cx),
                FieldEvent::Tab => this.open_notes(window, cx),
                FieldEvent::Cancel => this.dismiss(window, cx),
                _ => cx.notify(),
            }),
            cx.subscribe_in(&notes, window, |this: &mut Self, _, e, window, cx| match e {
                FieldEvent::Cancel => this.dismiss(window, cx),
                FieldEvent::BackTab => {
                    let f = this.title.read(cx).focus.clone();
                    window.focus(&f);
                }
                FieldEvent::Blur => {
                    // Emptied and left, the notes close again.
                    if this.notes.read(cx).text().trim().is_empty() {
                        this.notes_open = false;
                    }
                    cx.notify()
                }
                _ => cx.notify(),
            }),
        ];
        let f = title.read(cx).focus.clone();
        window.focus(&f);
        CaptureBar {
            db,
            system_dark,
            focus: cx.focus_handle(),
            title,
            notes,
            notes_open: false,
            hidden: false,
            shots: vec![],
            shot: Shot::Idle,
            anims: Anims::default(),
            focus_by_part: Default::default(),
            _shot_timer: None,
            _subs: subs,
            on_dismiss: None,
            on_open_main: None,
        }
    }

    fn theme(&self) -> Theme {
        let dark = match self.db.setting("theme").as_deref() {
            Some("dark") => true,
            Some("light") => false,
            _ => self.system_dark,
        };
        Theme::of(dark)
    }

    fn fh(&mut self, name: &str, cx: &mut Context<Self>) -> FocusHandle {
        self.focus_by_part.entry(name.to_string()).or_insert_with(|| cx.focus_handle()).clone()
    }

    fn on(&self, cx: &Context<Self>, f: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static) -> crate::ui::Handler {
        let this = cx.entity().downgrade();
        Rc::new(move |window, cx| {
            this.update(cx, |s, cx| {
                f(s, window, cx);
                cx.notify();
            })
            .ok();
        })
    }

    /// The tray's `target_list`: the last list the main window had focused, else the first list,
    /// else a new list in the default project.
    fn target(&self) -> Option<List> {
        let store = &self.db.store;
        let lists = self.db.run(store.list_lists()).ok()?;
        if let Some(id) = self.db.setting("last_active_list") {
            if let Some(l) = lists.iter().find(|l| l.id == id) {
                return Some(l.clone());
            }
        }
        lists.into_iter().next()
    }

    fn target_label(&self) -> String {
        match self.target() {
            Some(l) => {
                let p = self.db.run(self.db.store.list_projects()).ok().and_then(|ps| ps.into_iter().find(|p| p.id == l.project_id)).map(|p| p.name).unwrap_or_default();
                format!("{p} / {}", l.name)
            }
            None => "New list".into(),
        }
    }

    fn has_draft(&self, cx: &App) -> bool {
        !self.title.read(cx).text().is_empty() || !self.notes.read(cx).text().is_empty() || !self.shots.is_empty()
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.title.update(cx, |f, cx| f.set_text("", cx));
        self.notes.update(cx, |f, cx| f.set_text("", cx));
        self.shots.clear();
        self.notes_open = false;
        let f = self.title.read(cx).focus.clone();
        window.focus(&f);
        cx.notify();
    }

    fn open_notes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_open = true;
        self.anims.reset("notes");
        let f = self.notes.read(cx).focus.clone();
        window.focus(&f);
        cx.notify();
    }

    /// Files the task where the footer says, with its notes and screenshots.
    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let title = self.title.read(cx).text().trim().to_string();
        if title.is_empty() {
            return;
        }
        let list = match self.target() {
            Some(l) => l,
            None => {
                let Ok(p) = self.db.run(self.db.store.default_project()) else { return };
                let Ok(l) = self.db.run(self.db.store.create_list(&p.id, &taskscape_common::names::suggest_list_name())) else { return };
                l
            }
        };
        let store = &self.db.store;
        let Ok(t) = self.db.run(store.create_task(&list.id, &title, None, None)) else { return };
        report(cx, "taskCreated", json!({ "id": t.id, "title": t.title, "listId": t.list_id, "parentId": "" }));
        let notes = self.notes.read(cx).text().trim().to_string();
        if !notes.is_empty() {
            let html = crate::model::note_html(&notes);
            if self.db.run(store.create_note(&t.id, &html)).is_ok() {
                report(cx, "noteAdded", json!({ "taskId": t.id, "text": strip_html(&html) }));
            }
        }
        let multiple = self.shots.len() > 1;
        for (i, path) in self.shots.clone().iter().enumerate() {
            let name = if multiple { format!("screenshot-{}.png", i + 1) } else { "screenshot.png".into() };
            if let Ok(a) = self.db.run(taskscape_common::attachments::attach_copy(store, &t.id, path, Some(&name))) {
                report(cx, "attachmentAdded", json!({ "taskId": a.task_id, "name": a.name }));
            }
        }
        self.clear(window, cx);
    }

    /// The global shortcut's summons: the bar shows again, with the draft as it was left, focused
    /// on the title.
    pub fn summon(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.hidden = false;
        let f = self.title.read(cx).focus.clone();
        window.focus(&f);
        cx.notify();
    }

    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.hidden = true;
        cx.notify();
        report(cx, "dismissed", json!({}));
        if let Some(d) = self.on_dismiss.clone() {
            d(window, cx);
        }
    }

    fn screenshot(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        report(cx, "screenshotRequested", json!({}));
        self.shot = Shot::Capturing;
        cx.notify();
        // macOS's screencapture; elsewhere the app's own capture says it can't.
        match taskscape_common::screenshot::capture() {
            Ok(Some(path)) => {
                self.shots.push(path.display().to_string());
                self.shot = Shot::Idle;
            }
            Ok(None) => self.shot = Shot::Idle,
            Err(e) => {
                eprintln!("[taskscape] screenshot: {e:#}");
                self.shot = Shot::Failed;
                self._shot_timer = Some(cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(Duration::from_secs(4)).await;
                    this.update(cx, |s, cx| {
                        s.shot = Shot::Idle;
                        cx.notify()
                    })
                    .ok();
                }));
            }
        }
        cx.notify();
    }
}

impl Focusable for CaptureBar {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for CaptureBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();
        if self.hidden {
            return parts::frame(div().id("capture").key_context("Capture").track_focus(&self.focus).size_full().bg(t.surface_2));
        }
        for f in [&self.title, &self.notes] {
            f.update(cx, |f, _| {
                f.caret = t.accent_500;
                f.placeholder_color = t.content_3;
                f.selection_color = t.selection_1;
            });
        }
        let tfh = self.title.read(cx).focus.clone();
        let nfh = self.notes.read(cx).focus.clone();
        let draft = self.has_draft(cx);
        let glyphs = |id: &str| self.db.hotkeys().into_iter().find(|b| b.id == id).and_then(|b| b.accel).map(|a| a.glyphs()).unwrap_or_default();
        let (clear_keys, shot_keys) = (glyphs("clear_draft"), glyphs("screenshot_capture"));
        let mut title_row = div()
            .h(px(40.))
            .px(px(12.))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .child(part("captureTitle", "textbox").text(self.title.read(cx).text()).name("Capture a task ...").focus(&tfh, window).wrap(div().flex_1().text_size(px(14.)).line_height(px(20.)).child(self.title.clone())));
        if draft {
            let fh = self.fh("captureClear", cx);
            title_row = title_row.child(
                part("captureClear", "button").text(format!("Clear {clear_keys}")).focus(&fh, window).wrap(
                    pressable("captureClear", &fh, self.on(cx, |s, w, cx| s.clear(w, cx)))
                        .h(px(24.))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .gap(px(3.))
                        .rounded(px(8.))
                        .text_size(px(13.))
                        .text_color(t.content_3)
                        .hover(|s| s.bg(t.wash_2))
                        .child(icon("delete_sweep", 14., t.content_3))
                        .child("Clear")
                        .child(div().text_size(px(11.)).text_color(t.content_3).child(clear_keys.clone())),
                ),
            );
        }
        let mut card = div().w(px(480.)).flex().flex_col().rounded(px(12.)).bg(t.surface_2).border_1().border_color(t.edge_2).overflow_hidden().child(title_row);
        if self.notes_open {
            // The notes open at once: the app's tray has no transition on them (DESIGN.md's
            // "grid-row transition" is gone from its code).
            let p = 1.0f32;
            card = card.child(
                div().border_t_1().border_color(t.edge_1).px(px(12.)).py(px(10.)).min_h(px(64. * p)).max_h(px(160. * p)).overflow_hidden().opacity(p).child(
                    part("captureNotes", "textbox").text(self.notes.read(cx).text()).name("Notes ...").focus(&nfh, window).wrap(div().w_full().text_size(px(14.)).line_height(px(20.)).text_color(t.content_2).child(self.notes.clone())),
                ),
            );
        }
        let (fh_target, fh_shot, fh_hint) = (self.fh("captureTarget", cx), self.fh("captureShot", cx), self.fh("captureNotesHint", cx));
        let target = self.target_label();
        let shot_text = match (&self.shot, self.shots.len()) {
            (Shot::Capturing, _) => "Capturing …".to_string(),
            (Shot::Failed, _) => "Capture failed".to_string(),
            (Shot::Idle, 0) => "Screenshot".to_string(),
            (Shot::Idle, n) => format!("{n} shot{}", if n == 1 { "" } else { "s" }),
        };
        let mut footer = div()
            .h(px(40.))
            .px(px(8.))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .border_t_1()
            .border_color(t.edge_1)
            .child(
                part("captureTarget", "button").text(target.clone()).focus(&fh_target, window).wrap(
                    pressable("captureTarget", &fh_target, self.on(cx, |s, w, cx| {
                        report(cx, "mainOpened", json!({}));
                        if let Some(f) = s.on_open_main.clone() {
                            f(w, cx);
                        }
                    }))
                    .h(px(26.))
                    .px(px(6.))
                    .flex()
                    .items_center()
                    .gap(px(5.))
                    .rounded(px(8.))
                    .text_size(px(12.5))
                    .text_color(t.content_2)
                    .hover(|s| s.bg(t.wash_2))
                    .child(icon("open_in_new", 14., t.content_3))
                    .child(target),
                ),
            )
            .child(div().flex_1());
        if !self.notes_open {
            footer = footer.child(
                part("captureNotesHint", "button").text("Press Tab to add a note").focus(&fh_hint, window).wrap(
                    pressable("captureNotesHint", &fh_hint, self.on(cx, |s, w, cx| s.open_notes(w, cx))).text_size(px(12.)).text_color(t.content_3).child("Press Tab to add a note"),
                ),
            );
        }
        let failed = self.shot == Shot::Failed;
        footer = footer.child(
            part("captureShot", "button").text(format!("{shot_text} {shot_keys}")).name("Screenshot").focus(&fh_shot, window).wrap(
                pressable("captureShot", &fh_shot, self.on(cx, |s, w, cx| s.screenshot(w, cx)))
                    .h(px(26.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(5.))
                    .rounded(px(8.))
                    .text_size(px(12.5))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if failed { t.danger_500 } else { t.content_2 })
                    .hover(|s| s.bg(t.wash_2))
                    .child(icon("screenshot_monitor", 15., if failed { t.danger_500 } else { t.content_3 }))
                    .child(shot_text.clone())
                    .child(div().text_size(px(11.)).text_color(t.content_3).child(shot_keys.clone())),
            ),
        );
        card = card.child(footer);
        let root = div()
            .id("capture")
            .key_context("Capture")
            .track_focus(&self.focus)
            .size_full()
            .font_family(crate::system_font())
            .text_color(t.content_1)
            // The pop-up is sized to the card in the app; anything around it is the card's colour.
            .bg(t.surface_2)
            .on_action(cx.listener(|s, _: &CaptureSubmit, w, cx| s.submit(w, cx)))
            .on_action(cx.listener(|s, _: &ClearDraft, w, cx| s.clear(w, cx)))
            .on_action(cx.listener(|s, _: &Dismiss, w, cx| s.dismiss(w, cx)))
            .child(part("captureBar", "generic").wrap(card));
        parts::frame(root)
    }
}
