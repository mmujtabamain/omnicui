//! contract: omnicui/contracts/capturebar.contract.yaml (id: app.capturebar). The floating quick-capture bar: a title field, notes opened with Tab, screenshots, and the project and list a capture lands in.
use crate::App;
use egui::{pos2, vec2, Align, Color32, CornerRadius, Event, FontFamily, FontId, Key, Layout, Margin, Rect, Response, RichText, Role, Sense, Shape, Stroke, StrokeKind, TextEdit, WidgetInfo};
use taskscape_ui::omnicui::capturebar::{parts, CapturebarEvent, CapturebarProps};
use taskscape_ui::omnicui::charactercounter::{parts as counter_parts, CharactercounterProps};
use taskscape_ui::omnicui::hotkeyhint::HotkeyhintProps;
use taskscape_ui::omnicui::spinner::{parts as spinner_parts, SpinnerProps};
use taskscape_ui::omnicui::toggle::{parts as toggle_parts, ToggleProps};
use taskscape_ui::widgets::charactercounter::CharacterCounter;
use taskscape_ui::widgets::hotkeyhint::Hotkeyhint;
use taskscape_ui::widgets::spinner::Spinner;
use taskscape_ui::widgets::toggle::Toggle;
use taskscape_ui::{as_part, icon, part, rem, weight, Look, Mounted};

const TITLE_ID: &str = "app.capturebar.title.field";
const NOTES_ID: &str = "app.capturebar.notes.field";
const CLEAR_ID: &str = "app.capturebar.clear.button";
const HINT_ID: &str = "app.capturebar.notesHint.button";
const SHOT_ID: &str = "app.capturebar.screenshot.button";
/// The notes limit the counter reads against (count/2000).
const LIMIT: i64 = 2000;

/// Where a focus request lands, applied once the frame's widgets exist.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Focus {
    Title,
    Notes,
}

/// What a click on the bar asked for this frame.
#[derive(Default)]
struct Clicks {
    clear: bool,
    shot: bool,
    hint: bool,
    target: bool,
    toggled: bool,
}

/// The project and list the capture lands in, by name (empty where there is none).
struct Names {
    project: String,
    list: String,
}

impl Names {
    fn of(app: &App) -> Self {
        let project = app.ui.project.as_ref().and_then(|p| app.data.project(p)).map(|p| p.name.clone()).unwrap_or_default();
        let list = app.active_list().and_then(|l| app.data.list(&l).map(|l| l.name.clone())).unwrap_or_default();
        Self { project, list }
    }

    /// The footer's text: "Home / Errands", the project alone, or the fallback name.
    fn label(&self) -> String {
        match (self.project.is_empty(), self.list.is_empty()) {
            (false, false) => format!("{} / {}", self.project, self.list),
            (false, true) => self.project.clone(),
            (true, false) => self.list.clone(),
            (true, true) => "Taskscape".into(),
        }
    }
}

/// The capture bar's own state: its draft, what the screenshot button is doing, and the formatting
/// toggles. The bar is one small window, so `show` draws only its card.
pub struct CaptureBar {
    title: String,
    notes: String,
    notes_open: bool,
    /// The files of the screenshots taken (an empty name where a test answered without a file).
    shots: Vec<String>,
    /// Grabs in flight: 0 when idle, 1 while one is pending.
    pending: i64,
    /// "Capture failed" until the next capture starts.
    failed: bool,
    visible: bool,
    /// "native" takes the screen grab; a test's props answer with captured, cancelled, failed or pending.
    answer: String,
    bold: Toggle,
    italic: Toggle,
    underline: Toggle,
    strike: Toggle,
    bullets: Toggle,
    numbered: Toggle,
    /// The draft last reported, so draftChanged fires on a change only.
    last: (String, String, i64),
    /// A screenshot was asked for: draftChanged fires even when the draft itself did not change.
    touched: bool,
    focus: Option<Focus>,
    notes_had_focus: bool,
}

impl CaptureBar {
    pub fn new() -> Self {
        let toggle = |icon: &str, name: &str, chord: &str| {
            Toggle::new(
                ToggleProps { icon: icon.into(), ariaLabel: name.into(), shortcut: chord.into(), size: "md".into(), retainFocus: true, ..Default::default() },
                false,
            )
        };
        Self {
            title: String::new(),
            notes: String::new(),
            notes_open: false,
            shots: vec![],
            pending: 0,
            failed: false,
            visible: true,
            answer: "native".into(),
            bold: toggle("format_bold", "Bold", "Mod+B"),
            italic: toggle("format_italic", "Italic", "Mod+I"),
            underline: toggle("format_underlined", "Underline", "Mod+U"),
            strike: toggle("strikethrough_s", "Strikethrough", ""),
            bullets: toggle("format_list_bulleted", "Bulleted list", ""),
            numbered: toggle("format_list_numbered", "Numbered list", ""),
            last: (String::new(), String::new(), 0),
            touched: false,
            // The title takes focus when the bar appears.
            focus: Some(Focus::Title),
            notes_had_focus: false,
        }
    }

    /// One frame: draws the bar on the app and returns the events it raised in this frame.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<CapturebarEvent> {
        let mut ev = vec![];
        if !self.visible {
            return ev;
        }
        app.capture_open = true;
        let title_id = egui::Id::new(TITLE_ID);
        let notes_id = egui::Id::new(NOTES_ID);
        if self.focus == Some(Focus::Title) {
            ui.memory_mut(|m| m.request_focus(title_id));
        }

        // Keys first: the editors below must not see the keys the bar keeps.
        let title_focus = ui.memory(|m| m.has_focus(title_id));
        let notes_focus = self.notes_open && ui.memory(|m| m.has_focus(notes_id));
        if take_key(ui, Key::Escape, false, false) {
            self.hide(app, &mut ev);
            ui.ctx().request_repaint();
            return ev;
        }
        let cmd_enter = take_key(ui, Key::Enter, true, false);
        let plain_enter = title_focus && take_key(ui, Key::Enter, false, false);
        if title_focus {
            // Shift+Enter in the title does nothing: it is taken, so it never reaches the field.
            take_key(ui, Key::Enter, false, true);
        }
        let tab = (title_focus || notes_focus) && take_key(ui, Key::Tab, false, false);
        let clear_chord = take_key(ui, Key::Backspace, true, true);

        if (cmd_enter || plain_enter) && self.submit(app, &mut ev) {
            ui.ctx().request_repaint();
            return ev;
        }
        if tab {
            self.notes_open = true;
            self.focus = Some(Focus::Notes);
        }
        if clear_chord {
            self.clear_draft();
        }

        let names = Names::of(app);
        let card = egui::Frame::new()
            .fill(look.t.surface_2)
            .stroke(Stroke::new(1.0, look.t.edge_2))
            .corner_radius(CornerRadius::same(10));
        let out = card.show(ui, |ui| self.body(ui, look, &names));
        out.response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Quick capture"));
        part(ui, &out.response, parts::CARD);
        let c = out.inner;

        if c.clear {
            self.clear_draft();
        }
        if c.shot {
            self.capture(&mut ev);
        }
        if c.hint {
            self.notes_open = true;
            self.focus = Some(Focus::Notes);
        }
        if c.toggled {
            // A formatting button keeps the notes focused, as the web's toolbar keeps its blur off.
            self.focus = Some(Focus::Notes);
        }

        // Empty notes fold back to the hint when focus leaves them.
        let notes_now = ui.memory(|m| m.has_focus(notes_id));
        if self.notes_had_focus && !notes_now && self.notes_open && self.notes.is_empty() && self.focus != Some(Focus::Notes) {
            self.notes_open = false;
            ui.ctx().request_repaint();
        }
        self.notes_had_focus = notes_now;

        if c.target {
            ev.push(CapturebarEvent::MainRequested { project: names.project.clone(), list: names.list.clone() });
            self.hide(app, &mut ev);
            ui.ctx().request_repaint();
            return ev;
        }

        self.sync(&mut ev);
        match self.focus.take() {
            Some(Focus::Title) => ui.memory_mut(|m| m.request_focus(title_id)),
            Some(Focus::Notes) if self.notes_open => ui.memory_mut(|m| m.request_focus(notes_id)),
            _ => {}
        }
        ev
    }

    /// The card's rows: the title row, the notes (the hint, or the editor and its toolbar), and the footer.
    fn body(&mut self, ui: &mut egui::Ui, look: Look, names: &Names) -> Clicks {
        let t = look.t;
        let mut c = Clicks::default();
        // The rows are laid out at their exact heights, so nothing adds spacing between them.
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        let w = ui.available_width();

        // Title row (40 px): the Clear button, when there is a draft, sits at the right edge and the field takes the rest.
        let row = ui.allocate_ui_with_layout(vec2(w, 40.0), Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(12.0);
            if self.has_draft() {
                c.clear = clear_button(ui, look);
            }
            let field = (ui.available_width() - 12.0).max(0.0);
            let edit = TextEdit::singleline(&mut self.title)
                .id(egui::Id::new(TITLE_ID))
                .lock_focus(true)
                .frame(egui::Frame::NONE)
                .font(FontId::new(rem(1.0), weight(400)))
                .text_color(t.content_1)
                .hint_text(RichText::new("Capture a task ...").color(t.content_3))
                .desired_width(field);
            let r = ui.add(edit);
            part(ui, &r, parts::TITLE);
            ui.add_space(12.0);
        });
        tag_area(ui, row.response.rect, "app.capturebar.titleRow.area", parts::TITLEROW, "");
        rule(ui, t.edge_1);

        if self.notes_open {
            self.notes(ui, look);
        } else {
            self.hint(ui, look, &mut c);
        }
        rule(ui, t.edge_1);

        // Footer (40 px): the target at the left, the screenshot button at the right.
        let footer = ui.allocate_ui_with_layout(vec2(w, 40.0), Layout::left_to_right(Align::Center), |ui| {
            ui.add_space(12.0);
            c.target = target_button(ui, look, names).clicked();
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(12.0);
                c.shot = self.screenshot(ui, look);
            });
        });
        tag_area(ui, footer.response.rect, "app.capturebar.footer.area", parts::FOOTER, "");
        c
    }

    /// The open notes: the editor (64 px at least, growing to 160 px), then the formatting toolbar.
    fn notes(&mut self, ui: &mut egui::Ui, look: Look) {
        let t = look.t;
        let w = ui.available_width();
        egui::Frame::new().inner_margin(Margin::symmetric(12, 8)).show(ui, |ui| {
            egui::ScrollArea::vertical().max_height(144.0).min_scrolled_height(48.0).auto_shrink([false, true]).show(ui, |ui| {
                let edit = TextEdit::multiline(&mut self.notes)
                    .id(egui::Id::new(NOTES_ID))
                    .lock_focus(true)
                    .frame(egui::Frame::NONE)
                    .font(FontId::new(rem(0.964), weight(400)))
                    .text_color(t.content_1)
                    .hint_text(RichText::new("Notes ...").color(t.content_3))
                    .desired_width(ui.available_width())
                    .desired_rows(1);
                let r = ui.add(edit);
                part(ui, &r, parts::NOTES);
            });
        });

        let bar = ui.allocate_ui_with_layout(vec2(w, 37.0), Layout::left_to_right(Align::Center), |ui| {
            ui.add_space(8.0);
            let mut toggled = false;
            toggled |= toggle_part(ui, look, &mut self.bold, parts::BOLD);
            ui.add_space(4.0);
            toggled |= toggle_part(ui, look, &mut self.italic, parts::ITALIC);
            ui.add_space(4.0);
            toggled |= toggle_part(ui, look, &mut self.underline, parts::UNDERLINE);
            ui.add_space(4.0);
            toggled |= toggle_part(ui, look, &mut self.strike, parts::STRIKETHROUGH);
            ui.add_space(8.0);
            vline(ui, t.edge_1);
            ui.add_space(8.0);
            toggled |= toggle_part(ui, look, &mut self.bullets, parts::BULLETLIST);
            ui.add_space(4.0);
            toggled |= toggle_part(ui, look, &mut self.numbered, parts::NUMBEREDLIST);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(8.0);
                let count = self.notes.chars().count() as i64;
                as_part(ui, &[(counter_parts::READOUT, parts::COUNTER)], |ui| {
                    let _ = CharacterCounter::new(CharactercounterProps { count, limit: LIMIT, ..Default::default() }).ui(ui, look);
                });
                ui.add_space(8.0);
                vline(ui, t.edge_1);
            });
            toggled
        });
        let rect = bar.response.rect;
        ui.painter().rect_filled(Rect::from_min_size(rect.min, vec2(w, 1.0)), CornerRadius::ZERO, t.edge_1);
        tag_area(ui, rect, "app.capturebar.toolbar.area", parts::TOOLBAR, "Formatting");
        if bar.inner {
            self.focus = Some(Focus::Notes);
        }
    }

    /// The collapsed notes: the Tab hint, 32 px high.
    fn hint(&mut self, ui: &mut egui::Ui, look: Look, c: &mut Clicks) {
        let t = look.t;
        let text = "Press Tab to add a note";
        let font = FontId::new(rem(0.857), weight(400));
        let width = ui.painter().layout_no_wrap(text.to_owned(), font.clone(), t.content_3).size().x + 24.0;
        let (rect, resp) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click());
        let ink = if resp.hovered() { t.content_2 } else { t.content_3 };
        if resp.hovered() {
            ui.painter().rect_filled(rect, CornerRadius::same(4), t.wash_1);
        }
        let galley = ui.painter().layout_no_wrap(text.to_owned(), font, ink);
        let y = rect.center().y - galley.size().y / 2.0;
        ui.painter().galley(pos2(rect.left() + 12.0, y), galley, ink);
        if resp.has_focus() {
            ui.painter().rect_stroke(rect.expand(2.0), CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, text));
        part(ui, &resp, parts::NOTESHINT);
        if resp.clicked() {
            c.hint = true;
        }
    }

    /// The screenshot button: idle with its count, busy with a spinner, or failed. True when it was clicked.
    fn screenshot(&self, ui: &mut egui::Ui, look: Look) -> bool {
        let t = look.t;
        let capturing = self.pending > 0;
        let n = self.shots.len();
        let filled = !capturing && !self.failed && n > 0;
        let label = if capturing {
            "Capturing …".to_owned()
        } else if self.failed {
            "Capture failed".to_owned()
        } else if n == 0 {
            "Screenshot".to_owned()
        } else if n == 1 {
            "1 shot".to_owned()
        } else {
            format!("{n} shots")
        };
        let id = egui::Id::new(SHOT_ID);
        let hot = was_hovered(ui, id) && !capturing;
        let ink = if self.failed {
            t.danger_500
        } else if filled || hot {
            t.content_1
        } else {
            t.content_2
        };
        let bg = ui.painter().add(Shape::Noop);
        let inner = ui
            .horizontal_centered(|ui| {
                ui.add_space(8.0);
                if capturing {
                    as_part(ui, &[(spinner_parts::INDICATOR, parts::SPINNER)], |ui| {
                        let _ = Spinner::new(SpinnerProps { size: "sm".into(), label: "Capturing".into(), ..Default::default() }).ui(ui, look);
                    });
                } else if self.failed {
                    glyph(ui, "error", 15.0, ink);
                } else {
                    glyph(ui, "screenshot_monitor", 15.0, ink);
                }
                ui.add_space(4.0);
                words(ui, &label, 12.0, 400, ink);
                if !capturing && !self.failed {
                    ui.add_space(6.0);
                    let _ = Hotkeyhint::new(HotkeyhintProps { accel: chord("Enter"), size: "sm".into(), tone: "muted".into(), ..Default::default() }).ui(ui, look);
                }
                ui.add_space(8.0);
            })
            .response
            .rect;
        let rect = Rect::from_center_size(inner.center(), vec2(inner.width(), 24.0));
        let sense = if capturing { Sense::hover() } else { Sense::click() };
        let resp = ui.interact(rect, id, sense);
        remember_hover(ui, id, resp.hovered());
        let fill = if resp.hovered() && !capturing {
            t.wash_1
        } else if filled {
            t.wash_2
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().set(bg, Shape::rect_filled(rect, CornerRadius::same(8), fill));
        resp.widget_info(|| WidgetInfo::labeled(Role::Button, !capturing, label.as_str()));
        part(ui, &resp, parts::SCREENSHOT);
        resp.clicked()
    }

    fn has_draft(&self) -> bool {
        !self.title.is_empty() || !self.notes.is_empty() || !self.shots.is_empty()
    }

    fn clear_draft(&mut self) {
        self.title.clear();
        self.notes.clear();
        self.notes_open = false;
        self.shots.clear();
        self.failed = false;
        self.focus = Some(Focus::Title);
    }

    /// One screenshot request. The grab resolves at once (or stays pending, for a test that holds it).
    fn capture(&mut self, ev: &mut Vec<CapturebarEvent>) {
        if self.pending > 0 {
            return;
        }
        self.touched = true;
        self.failed = false;
        self.pending += 1;
        ev.push(CapturebarEvent::CaptureStarted { pending: self.pending });
        let outcome = match self.answer.as_str() {
            "native" => match taskscape_storage::screenshot::capture() {
                Ok(Some(path)) => Outcome::Shot(path.to_string_lossy().into_owned()),
                Ok(None) => Outcome::Cancelled,
                Err(_) => Outcome::Failed,
            },
            "captured" => Outcome::Shot(String::new()),
            "cancelled" => Outcome::Cancelled,
            "failed" => Outcome::Failed,
            _ => Outcome::InFlight,
        };
        match outcome {
            Outcome::Shot(file) => {
                self.pending -= 1;
                self.shots.push(file);
                self.focus = Some(Focus::Title);
            }
            Outcome::Cancelled => {
                self.pending -= 1;
                self.focus = Some(Focus::Title);
            }
            Outcome::Failed => {
                self.pending -= 1;
                self.failed = true;
            }
            Outcome::InFlight => {}
        }
    }

    /// Files the draft as a task in the target list, attaching the screenshots. Returns false when
    /// there was nothing to save or the save was refused (the draft then stays as it is).
    fn submit(&mut self, app: &mut App, ev: &mut Vec<CapturebarEvent>) -> bool {
        if self.title.trim().is_empty() && self.notes.trim().is_empty() && self.shots.is_empty() {
            return false;
        }
        let Some(project_id) = app.ui.project.clone() else {
            app.refused(anyhow::anyhow!("There is no project to capture into"));
            return false;
        };
        let list_id = match app.active_list().filter(|l| app.data.list(l).is_some()) {
            Some(l) => l,
            None => match app.data.lists_of(&project_id).first().map(|l| l.id.clone()) {
                Some(l) => l,
                None => match app.data.create_list(&project_id, "Inbox") {
                    Ok(l) => l.id,
                    Err(e) => {
                        app.refused(e);
                        return false;
                    }
                },
            },
        };
        let project = app.data.project(&project_id).map(|p| p.name.clone()).unwrap_or_default();
        let list = app.data.list(&list_id).map(|l| l.name.clone()).unwrap_or_default();
        let title = if self.title.trim().is_empty() { "Untitled".to_owned() } else { self.title.trim().to_owned() };
        let notes = self.notes.trim();
        let task = match app.data.create_task(&list_id, &title, (!notes.is_empty()).then_some(notes), None) {
            Ok(t) => t,
            Err(e) => {
                app.refused(e);
                return false;
            }
        };
        for file in self.shots.iter().filter(|f| !f.is_empty()) {
            if let Err(e) = app.data.add_file(&task.id, file) {
                app.refused(e);
            }
        }
        ev.push(CapturebarEvent::TaskSaved {
            title: self.title.clone(),
            notes: self.notes.clone(),
            shots: self.shots.len() as i64,
            project,
            list,
        });
        self.title.clear();
        self.notes.clear();
        self.notes_open = false;
        self.shots.clear();
        self.failed = false;
        self.sync(ev);
        self.hide(app, ev);
        true
    }

    /// Hides the bar and keeps its draft for the next reveal.
    fn hide(&mut self, app: &mut App, ev: &mut Vec<CapturebarEvent>) {
        ev.push(CapturebarEvent::BarHidden { title: self.title.clone(), notes: self.notes.clone() });
        self.visible = false;
        app.capture_open = false;
    }

    fn tuple(&self) -> (String, String, i64) {
        (self.title.clone(), self.notes.clone(), self.shots.len() as i64)
    }

    /// draftChanged, when the draft differs from the last one reported, or a screenshot was asked for.
    fn sync(&mut self, ev: &mut Vec<CapturebarEvent>) {
        let now = self.tuple();
        if now != self.last || self.touched {
            ev.push(CapturebarEvent::DraftChanged { title: now.0.clone(), notes: now.1.clone(), shots: now.2 });
            self.last = now;
        }
        self.touched = false;
    }
}

enum Outcome {
    Shot(String),
    Cancelled,
    Failed,
    InFlight,
}

/// The Command-Shift chord for a key, in the names the hint widget reads: Command on macOS, Control elsewhere
/// (the contract's clearDraft default and the reference's screenshot chord).
fn chord(key: &str) -> String {
    let platform = if cfg!(target_os = "macos") { "Meta" } else { "Control" };
    format!("{platform}+Shift+{key}")
}

/// Takes one key press with exactly these modifiers: the platform modifier (Command on macOS, Control
/// elsewhere, either flag counts) when `platform` is set, and Shift when `shift` is. Other modifiers
/// fail the match, so Enter does not take Shift+Enter.
fn take_key(ui: &egui::Ui, key: Key, platform: bool, shift: bool) -> bool {
    ui.input_mut(|i| {
        let at = i.events.iter().position(|e| match e {
            Event::Key { key: k, modifiers: m, pressed: true, .. } => {
                let cmd = if cfg!(target_os = "macos") { m.command || m.mac_cmd } else { m.ctrl || m.command };
                *k == key && cmd == platform && m.shift == shift && !m.alt
            }
            _ => false,
        });
        match at {
            Some(at) => {
                i.events.remove(at);
                true
            }
            None => false,
        }
    })
}

/// The Clear button: a well at the right edge of the title row. True when it was clicked.
fn clear_button(ui: &mut egui::Ui, look: Look) -> bool {
    let t = look.t;
    let id = egui::Id::new(CLEAR_ID);
    let hot = was_hovered(ui, id);
    let ink = if hot { t.content_1 } else { t.content_2 };
    let bg = ui.painter().add(Shape::Noop);
    let inner = ui
        .horizontal_centered(|ui| {
            ui.add_space(8.0);
            glyph(ui, "delete_sweep", 16.0, ink);
            ui.add_space(2.0);
            words(ui, "Clear", 11.0, 400, ink);
            ui.add_space(6.0);
            let _ = Hotkeyhint::new(HotkeyhintProps { accel: chord("Backspace"), size: "sm".into(), tone: "muted".into(), ..Default::default() }).ui(ui, look);
            ui.add_space(8.0);
        })
        .response
        .rect;
    let rect = Rect::from_center_size(inner.center(), vec2(inner.width(), 25.0));
    let resp = ui.interact(rect, id, Sense::click());
    remember_hover(ui, id, resp.hovered());
    let fill = if resp.hovered() { t.wash_1 } else { Color32::TRANSPARENT };
    ui.painter().set(bg, Shape::rect_filled(rect, CornerRadius::same(8), fill));
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Clear"));
    part(ui, &resp, parts::CLEAR);
    resp.clicked()
}

/// The footer's target: the project, a muted slash and the list, with the open-in icon. Opens the main window.
fn target_button(ui: &mut egui::Ui, look: Look, names: &Names) -> Response {
    let t = look.t;
    let font = FontId::new(rem(0.857), weight(400));
    let icon_font = FontId::new(14.0, FontFamily::Name("Icons".into()));
    let glyph_text = icon("open_in_new", 14.0).text().to_owned();
    // The pieces in order: the project, the slash (the muted one), the list; or the fallback name alone.
    let mut pieces: Vec<(String, bool)> = vec![];
    if !names.project.is_empty() {
        pieces.push((names.project.clone(), false));
    }
    if !names.project.is_empty() && !names.list.is_empty() {
        pieces.push(("/".into(), true));
    }
    if !names.list.is_empty() {
        pieces.push((names.list.clone(), false));
    }
    if pieces.is_empty() {
        pieces.push(("Taskscape".into(), false));
    }
    let mut width = 14.0 + 6.0;
    for (s, slash) in &pieces {
        let w = ui.painter().layout_no_wrap(s.clone(), font.clone(), t.content_2).size().x;
        width += w + if *slash { 8.0 } else { 0.0 };
    }
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 24.0), Sense::click());
    let ink = if resp.hovered() { t.content_1 } else { t.content_2 };
    let cy = rect.center().y;
    let painter = ui.painter();
    let g = painter.layout_no_wrap(glyph_text, icon_font, ink);
    painter.galley(pos2(rect.left(), cy - g.size().y / 2.0), g, ink);
    let mut x = rect.left() + 14.0 + 6.0;
    for (s, slash) in pieces {
        let color = if slash { ink.gamma_multiply(0.6) } else { ink };
        if slash {
            x += 4.0;
        }
        let g = painter.layout_no_wrap(s, font.clone(), color);
        let (w, h) = (g.size().x, g.size().y);
        painter.galley(pos2(x, cy - h / 2.0), g, color);
        x += w + if slash { 4.0 } else { 0.0 };
    }
    let label = names.label();
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, label.as_str()));
    part(ui, &resp, parts::TARGET);
    resp
}

/// One toggle of the formatting bar, its state kept by the widget, under the screen's part name.
fn toggle_part(ui: &mut egui::Ui, look: Look, toggle: &mut Toggle, part_id: &str) -> bool {
    !as_part(ui, &[(toggle_parts::TOGGLE, part_id)], |ui| toggle.ui(ui, look)).is_empty()
}

/// Last frame's hover of a control, so its ink can follow the pointer.
fn was_hovered(ui: &egui::Ui, id: egui::Id) -> bool {
    ui.ctx().data(|d| d.get_temp::<bool>(id)).unwrap_or(false)
}

fn remember_hover(ui: &egui::Ui, id: egui::Id, hovered: bool) {
    ui.ctx().data_mut(|d| d.insert_temp(id, hovered));
}

/// A 1px horizontal rule across the row.
fn rule(ui: &mut egui::Ui, color: Color32) {
    let width = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
    ui.painter().rect_filled(r, CornerRadius::ZERO, color);
}

/// A 1px vertical rule, 16px tall.
fn vline(ui: &mut egui::Ui, color: Color32) {
    let (r, _) = ui.allocate_exact_size(vec2(1.0, 16.0), Sense::hover());
    ui.painter().rect_filled(r, CornerRadius::ZERO, color);
}

/// A glyph from the icon font, drawn in one colour.
fn glyph(ui: &mut egui::Ui, name: &str, px: f32, color: Color32) {
    let font = FontId::new(px, FontFamily::Name("Icons".into()));
    let g = ui.painter().layout_no_wrap(icon(name, px).text().to_owned(), font, color);
    let (r, _) = ui.allocate_exact_size(g.size(), Sense::hover());
    ui.painter().galley(r.min, g, color);
}

/// A line of Inter text.
fn words(ui: &mut egui::Ui, s: &str, px: f32, w: u16, color: Color32) {
    let g = ui.painter().layout_no_wrap(s.to_owned(), FontId::new(px, weight(w)), color);
    let (r, _) = ui.allocate_exact_size(g.size(), Sense::hover());
    ui.painter().galley(r.min, g, color);
}

/// An invisible region under a row or group, tagged as a contract part.
fn tag_area(ui: &egui::Ui, rect: Rect, id: &str, part_id: &str, label: &str) {
    let r = ui.interact(rect, egui::Id::new(id), Sense::hover());
    r.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, label));
    part(ui, &r, part_id);
}

/// The test's mount: the app on the props' data set, the draft seeded and the screenshot answer set.
struct Mount {
    app: App,
    screen: CaptureBar,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

pub fn mount(app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: CapturebarProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    let mut screen = CaptureBar::new();
    screen.answer = p.captureResult.clone();
    match p.draft.as_str() {
        "typed" => screen.title = "Call the plumber".into(),
        "notes" => {
            screen.title = "Call the plumber".into();
            screen.notes_open = true;
        }
        _ => {}
    }
    // The seeded draft is not a change: nothing is reported for it.
    screen.last = screen.tuple();
    Ok(Box::new(Mount { app, screen }))
}
