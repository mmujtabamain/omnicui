//! contract: omnicui/contracts/sheets.contract.yaml (id: app.sheets). The one confirmation or prompt card the app's sheet shows (New list, rename, delete, Move to Trash, Empty Trash, Delete permanently) and the Moved to Trash toast with Undo; drawn as the content of its own window, with no backdrop.
use crate::app::Sheet;
use crate::App;
use egui::{
    pos2, vec2, Align, Align2, Area, Color32, CornerRadius, FontId, Frame, Id, Key, Layout, Margin, Modifiers, Order, Rect, Role, Sense, Shadow,
    Stroke, StrokeKind, UiBuilder, WidgetInfo,
};
use std::time::Duration;
use taskscape_ui::omnicui::button::{parts as button_parts, ButtonProps};
use taskscape_ui::omnicui::dialogbutton::{parts as dialogbutton_parts, DialogbuttonProps};
use taskscape_ui::omnicui::heading::{parts as heading_parts, HeadingProps};
use taskscape_ui::omnicui::icon::{parts as icon_parts, IconProps};
use taskscape_ui::omnicui::input::{parts as input_parts, InputEvent, InputProps};
use taskscape_ui::omnicui::sheets::{parts, SheetsEvent, SheetsProps};
use taskscape_ui::omnicui::text::TextProps;
use taskscape_ui::widgets::button::Button;
use taskscape_ui::widgets::dialogbutton::DialogButton;
use taskscape_ui::widgets::heading::Heading;
use taskscape_ui::widgets::icon::Icon;
use taskscape_ui::widgets::input::Input;
use taskscape_ui::widgets::text::Text;
use taskscape_ui::{as_part, part, rem, text, weight, Look, Mounted};

/// The card's width (26rem) and the lg footer button's height and label sides (the web's lg size).
pub const CARD_W: f32 = rem(26.0);
const BTN_H: f32 = rem(2.0);
const BTN_SIDE: f32 = 10.0;
/// The gap between the footer's buttons and between the name field's items (the web's 0.357rem).
const GAP: f32 = 5.0;
/// The toast: 6 seconds on screen, rising in over 0.28 s from 8 px lower.
const TOAST_SECS: f64 = 6.0;
const RISE_SECS: f64 = 0.28;
const TOAST_RISE: f32 = rem(0.571);

const LIST_FIRST: [&str; 12] = ["Golden", "Silver", "Quiet", "Gilded", "Misty", "Amber", "Coral", "Velvet", "Cobalt", "Sunny", "Maple", "Hollow"];
const LIST_SECOND: [&str; 10] = ["Oasis", "Harbor", "Meadow", "Lantern", "Orchard", "Canyon", "Lagoon", "Summit", "Garden", "Fjord"];
const PROJECT_NAMES: [&str; 12] = ["Nereid", "Titan", "Halcyon", "Orion", "Solace", "Atlas", "Meridian", "Cinder", "Tundra", "Zephyr", "Lumen", "Juniper"];

/// Which rolled name a prompt's dice button gives.
#[derive(Clone, Copy, PartialEq)]
enum Roll {
    List,
    Project,
}

/// A name prompt: its placeholder and whether it has the dice button.
struct Prompt {
    placeholder: &'static str,
    roll: Option<Roll>,
}

/// What a sheet shows: its glyph, tone, copy, accept label, and whether it has Import JSON and a name field.
struct Config {
    glyph: &'static str,
    danger: bool,
    title: String,
    detail: Option<String>,
    accept: &'static str,
    instead: bool,
    prompt: Option<Prompt>,
}

/// What the card's buttons and keys asked for this frame.
enum Act {
    Dismiss,
    Accept(String),
    Instead(String),
}

/// The screen's own state: the sheet it last drew, its name field, and the toast's timer.
pub struct Sheets {
    /// The text the next prompt starts with, selected, instead of its rolled or current name (a window
    /// that opens a prompt with a name already in it).
    pub initial_name: Option<String>,
    /// The sheet the card last drew. A change of `app.sheet` starts the card over.
    shown: Option<Sheet>,
    /// The name field of a prompt. It is rebuilt when the prompt starts and when the dice roll.
    input: Option<Input>,
    /// The name as the field holds it (raw; trimmed when accepted).
    name: String,
    /// The card's height last frame, so that it sits in the middle of its window.
    card_h: f32,
    /// Frames left in which a destructive sheet moves its focus onto Cancel.
    focus_frames: u8,
    /// The toast on screen, and when it appeared. A new toast restarts the timer.
    toast_key: Option<String>,
    toast_at: f64,
}

impl Default for Sheets {
    fn default() -> Self {
        Self::new()
    }
}

impl Sheets {
    /// The card's height in the last frame: a window holding only the card fits itself to it.
    pub fn card_height(&self) -> f32 {
        self.card_h
    }

    pub fn new() -> Self {
        Self { initial_name: None, shown: None, input: None, name: String::new(), card_h: 0.0, focus_frames: 0, toast_key: None, toast_at: 0.0 }
    }

    /// One frame, on the app: the card of `app.sheet` and the Moved to Trash toast. Returns the
    /// events the sheet raised (answered).
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<SheetsEvent> {
        let mut events = vec![];
        self.sync(app);
        if let Some(sheet) = app.sheet.clone() {
            // Escape cancels from anywhere in the card, before the name field reads it.
            let escape = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
            let cfg = config(app, &sheet);
            if let Some(act) = self.card(ui, look, &cfg, escape) {
                match act {
                    Act::Dismiss => {
                        app.sheet = None;
                        events.push(answered("dismiss", "", ""));
                    }
                    Act::Instead(id) => {
                        app.sheet = None;
                        events.push(answered("instead", "", &id));
                        // Import JSON: a list from an export file instead of a new empty one (the reference's).
                        if id == "import" {
                            app.import_list(None);
                        }
                    }
                    Act::Accept(name) => match commit(app, &sheet, &name) {
                        Ok(()) => {
                            app.sheet = None;
                            let text = if cfg.prompt.is_some() { name } else { String::new() };
                            events.push(answered("accept", &text, ""));
                        }
                        // The save was refused: the sheet stays up and the app's toast says why.
                        Err(e) => app.refused(e),
                    },
                }
            }
        }
        self.toast(app, ui, look);
        events
    }

    /// Starts the card over when `app.sheet` changed: the name field, its text and the focus.
    fn sync(&mut self, app: &App) {
        if app.sheet == self.shown {
            return;
        }
        self.shown = app.sheet.clone();
        self.input = None;
        self.name.clear();
        self.focus_frames = 0;
        let Some(sheet) = &app.sheet else { return };
        let cfg = config(app, sheet);
        let given = self.initial_name.take().filter(|n| !n.is_empty());
        if let (Some(prompt), Some(initial)) = (&cfg.prompt, given.or_else(|| prompt_initial(app, sheet))) {
            self.input = Some(make_input(&initial, &cfg.title, prompt.placeholder));
            self.name = initial;
        }
        // A destructive sheet opens with Cancel focused (decision 13).
        if cfg.danger {
            self.focus_frames = 6;
        }
    }

    /// The card: centred in the window, its body and footer. Returns what its controls asked for.
    fn card(&mut self, ui: &mut egui::Ui, look: Look, cfg: &Config, escape: bool) -> Option<Act> {
        let t = look.t;
        let mut act = if escape { Some(Act::Dismiss) } else { None };
        // Enter on a focused Accept of a destructive sheet does not confirm (decision 10).
        let enter = ui.input(|i| i.key_pressed(Key::Enter));

        let area = ui.max_rect();
        let card_w = CARD_W.min(area.width());
        let x = area.center().x - card_w / 2.0;
        let y = (area.center().y - self.card_h / 2.0).max(area.top());
        let slot = Rect::from_min_size(pos2(x, y), vec2(card_w, (area.bottom() - y).max(0.0)));

        let frame = Frame::new()
            .fill(t.surface_2)
            .stroke(Stroke::new(1.0, t.edge_1))
            .corner_radius(CornerRadius::same(10))
            .shadow(Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(64) });
        let rect = ui
            .scope_builder(UiBuilder::new().max_rect(slot), |ui| {
                frame
                    .show(ui, |ui| {
                        ui.set_width(card_w - 2.0);
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                        self.body(ui, look, cfg, enter, &mut act);
                    })
                    .response
                    .rect
            })
            .inner;
        self.card_h = rect.height();
        let node = ui.interact(rect, ui.scope_id().with("sheets.card"), Sense::hover());
        // The card reads as its headline and detail.
        let words = match &cfg.detail {
            Some(d) => format!("{} {}", cfg.title, d),
            None => cfg.title.clone(),
        };
        node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &words));
        part(ui, &node, parts::CARD);

        // A destructive sheet starts on Cancel until something else takes the focus (decision 13); see the footer.
        if self.focus_frames > 0 {
            if ui.ctx().memory(|m| m.focused()).is_some() {
                self.focus_frames = 0;
            } else {
                self.focus_frames -= 1;
                ui.ctx().request_repaint();
            }
        }
        // A name prompt keeps the focus in its field: a Tab that wrapped onto the dice button (which is
        // not in the Tab order) returns to the field.
        if self.input.is_some() && ui.ctx().memory(|m| m.focused()).is_none() {
            ui.ctx().memory_mut(|m| m.request_focus(input_id()));
        }
        act
    }

    /// The card's body (the glyph and headline, the detail line, the name field) and its footer.
    fn body(&mut self, ui: &mut egui::Ui, look: Look, cfg: &Config, enter: bool, act: &mut Option<Act>) {
        let t = look.t;
        Frame::new().inner_margin(Margin { left: 10, right: 10, top: 8, bottom: 10 }).show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = GAP;
                let mut glyph = Icon::new(IconProps {
                    name: cfg.glyph.into(),
                    tint: if cfg.danger { "danger" } else { "accent" }.into(),
                    ..Default::default()
                });
                as_part(ui, &[(icon_parts::GLYPH, parts::GLYPH)], |ui| glyph.ui(ui, look));
                let mut heading = Heading::new(HeadingProps {
                    r#as: "h2".into(),
                    size: "md".into(),
                    overflow: "wrap".into(),
                    content: cfg.title.clone(),
                });
                as_part(ui, &[(heading_parts::TITLE, parts::HEADLINE)], |ui| heading.ui(ui, look));
            });
            if let Some(detail) = &cfg.detail {
                ui.add_space(rem(0.429));
                let r = ui.label(text(detail.clone(), rem(0.857), 400).color(t.content_2));
                r.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, detail));
                part(ui, &r, parts::DETAIL);
            }
            if let Some(prompt) = &cfg.prompt {
                ui.add_space(rem(0.714));
                self.field(ui, look, cfg, prompt, act);
            }
        });
        self.footer(ui, look, cfg, enter, act);
    }

    /// The name field: the bordered row with the text input and, for the prompts that have one, the dice button.
    fn field(&mut self, ui: &mut egui::Ui, look: Look, cfg: &Config, prompt: &Prompt, act: &mut Option<Act>) {
        let t = look.t;
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), rem(2.5)), Sense::hover());
        // The row is the field: its border becomes the focus ring while the name has the keyboard.
        let stroke = if ui.ctx().memory(|m| m.has_focus(input_id())) { Stroke::new(2.0, t.focus_1) } else { Stroke::new(1.0, t.edge_2) };
        ui.painter().rect(rect, CornerRadius::same(5), t.surface_1, stroke, StrokeKind::Inside);
        // The row's padding: 10px at the left, 6px at the right.
        let inner = Rect::from_min_max(pos2(rect.left() + 10.0, rect.top()), pos2(rect.right() - 6.0, rect.bottom()));

        let mut roll_clicked = false;
        let mut submit: Option<String> = None;
        ui.scope_builder(UiBuilder::new().max_rect(inner).layout(Layout::right_to_left(Align::Center)), |ui| {
            ui.spacing_mut().item_spacing.x = GAP;
            // The dice button is first in the row, so it is first in the Tab order too; it is not tabbable.
            if prompt.roll.is_some() {
                let mut roll = Button::new(ButtonProps {
                    variant: "ghost".into(),
                    size: "md".into(),
                    shape: "icon".into(),
                    tone: "muted".into(),
                    icon: "casino".into(),
                    iconSize: 16,
                    ariaLabel: "Roll another name".into(),
                    tabbable: false,
                    ..Default::default()
                });
                roll_clicked = !as_part(ui, &[(button_parts::BUTTON, parts::ROLL)], |ui| roll.ui(ui, look)).is_empty();
            }
            if let Some(input) = self.input.as_mut() {
                for e in as_part(ui, &[(input_parts::FIELD, parts::NAMEFIELD)], |ui| input.ui(ui, look)) {
                    match e {
                        InputEvent::ValueChange { value } => self.name = value,
                        InputEvent::Submit { value } => submit = Some(value),
                        _ => {}
                    }
                }
            }
        });

        // A roll starts a new field with the rolled name, selected, and focused next frame.
        if roll_clicked {
            if let Some(kind) = prompt.roll {
                let value = roll_name(kind);
                self.input = Some(make_input(&value, &cfg.title, prompt.placeholder));
                self.name = value;
            }
        }
        if let Some(value) = submit {
            // Enter on an empty name refocuses the field; a name is accepted.
            if value.is_empty() {
                ui.ctx().memory_mut(|m| m.request_focus(input_id()));
            } else {
                *act = Some(Act::Accept(value));
            }
        }
    }

    /// The footer: a hairline, Import JSON at the left (newList), Cancel then Accept at the right.
    fn footer(&mut self, ui: &mut egui::Ui, look: Look, cfg: &Config, enter: bool, act: &mut Option<Act>) {
        let t = look.t;
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0 + 9.0 + BTN_H + 9.0), Sense::hover());
        ui.painter().rect_filled(Rect::from_min_size(rect.left_top(), vec2(rect.width(), 1.0)), CornerRadius::ZERO, t.edge_1);
        let cy = rect.top() + 10.0 + BTN_H / 2.0;

        // Buttons of the name sheets are at least 6rem wide, those of the deletions 7.143rem; the two
        // footer columns take the wider of the two buttons.
        let min_w = if cfg.danger { rem(7.143) } else { rem(6.0) };
        let cancel_c = lg_width(ui, "Cancel");
        let accept_c = lg_width(ui, cfg.accept);
        let col = cancel_c.max(accept_c).max(min_w);
        let group_left = rect.right() - 10.0 - (2.0 * col + GAP);
        // Each button fills its column (the web's min-w), its label centred.
        let cancel_r = Rect::from_min_size(pos2(group_left, cy - BTN_H / 2.0), vec2(col, BTN_H));
        let accept_r = Rect::from_min_size(pos2(group_left + col + GAP, cy - BTN_H / 2.0), vec2(col, BTN_H));


        if cfg.instead {
            let import_c = lg_width(ui, "Import JSON…").max(rem(6.0));
            let r = Rect::from_min_size(pos2(rect.left() + 10.0, cy - BTN_H / 2.0), vec2(import_c, BTN_H));
            let mut import = DialogButton::new(DialogbuttonProps {
                role: "instead".into(),
                insteadId: "import".into(),
                variant: "".into(),
                label: "Import JSON…".into(),
                disabled: false,
            });
            let pressed = !at(ui, "sheets.import", r, |ui| as_part(ui, &[(dialogbutton_parts::BUTTON, parts::IMPORTJSON)], |ui| import.ui(ui, look))).is_empty();
            if pressed {
                *act = Some(Act::Instead("import".into()));
            }
            outline(ui, look, r);
        }

        // Cancel is drawn here, with the look of the lg neutral Button, because a destructive sheet starts
        // with it focused (decision 13) and only a response of this control can be given that focus.
        // Enter and Space on the focused control click it, as they do a Button.
        let cancel_resp = ui.interact(cancel_r, Id::unique("sheets.cancel"), Sense::click());
        if cfg.danger && self.focus_frames > 0 && ui.ctx().memory(|m| m.focused()).is_none() {
            cancel_resp.request_focus();
        }
        if cancel_resp.clicked() {
            *act = Some(Act::Dismiss);
        }
        cancel_resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Cancel"));
        part(ui, &cancel_resp, parts::CANCEL);
        let cancel_fill = if cancel_resp.hovered() { t.wash_2 } else { t.surface_0 };
        ui.painter().rect_filled(cancel_r, CornerRadius::same(4), cancel_fill);
        ui.painter().text(cancel_r.center(), Align2::CENTER_CENTER, "Cancel", FontId::new(14.0, weight(500)), t.content_1);
        outline(ui, look, cancel_r);
        if cancel_resp.has_focus() {
            ui.painter().rect_stroke(cancel_r, CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }

        let prompt = cfg.prompt.is_some();
        let disabled = prompt && self.name.trim().is_empty();
        let mut accept = Button::new(ButtonProps {
            variant: if cfg.danger { "danger" } else { "neutral" }.into(),
            size: "lg".into(),
            width: "full".into(),
            label: cfg.accept.into(),
            disabled,
            ..Default::default()
        });
        let pressed = !at(ui, "sheets.accept", accept_r, |ui| as_part(ui, &[(button_parts::BUTTON, parts::ACCEPT)], |ui| accept.ui(ui, look))).is_empty();
        if !cfg.danger {
            outline(ui, look, accept_r);
        }
        if pressed && !disabled && !(cfg.danger && enter) {
            let name = if prompt { self.name.trim().to_owned() } else { String::new() };
            *act = Some(Act::Accept(name));
        }
    }

    /// The "Moved to Trash" toast (or a refused save's message), at the bottom centre, with Undo and a
    /// close control. It leaves after six seconds, and a new toast restarts the timer.
    fn toast(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) {
        let now = ui.input(|i| i.time);
        let Some(toast) = app.toast.clone() else {
            self.toast_key = None;
            return;
        };
        let key = format!("{}|{:?}|{:?}", toast.message, toast.undo, toast.error);
        if self.toast_key.as_deref() != Some(key.as_str()) {
            self.toast_key = Some(key);
            self.toast_at = now;
        }
        let age = now - self.toast_at;
        if age >= TOAST_SECS {
            app.toast = None;
            self.toast_key = None;
            return;
        }
        ui.ctx().request_repaint_after(Duration::from_secs_f64(TOAST_SECS - age));
        let progress = (age / RISE_SECS).clamp(0.0, 1.0) as f32;
        let rise = TOAST_RISE * (1.0 - progress).powi(3);

        let message = toast.error.clone().unwrap_or_else(|| toast.message.clone());
        // A refused save has nothing to undo; only the close control is offered.
        let has_undo = toast.error.is_none() && !toast.undo.is_empty();
        let t = look.t;
        let ctx = ui.ctx().clone();
        let (mut undo, mut close) = (false, false);
        Area::new(ui.scope_id().with("app.sheets.toast"))
            .order(Order::Foreground)
            .movable(false)
            .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -rem(1.429) + rise))
            .show(&ctx, |ui| {
                let frame = Frame::new()
                    .fill(t.surface_3)
                    .stroke(Stroke::new(1.0, t.edge_2))
                    .corner_radius(CornerRadius::same(7))
                    .shadow(Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(if look.dark { 110 } else { 40 }) })
                    .inner_margin(Margin { left: 12, right: 5, top: 0, bottom: 0 });
                let r = frame
                    .show(ui, |ui| {
                        ui.set_height(rem(2.929) - 2.0);
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = rem(0.5);
                            Text::new(TextProps { content: message.clone(), ..Default::default() }).ui(ui, look);
                            if has_undo {
                                let mut undo_btn = Button::new(ButtonProps {
                                    variant: "ghost".into(),
                                    tone: "strong".into(),
                                    hover: "wash".into(),
                                    size: "sm".into(),
                                    width: "content".into(),
                                    label: "Undo".into(),
                                    ..Default::default()
                                });
                                undo = !as_part(ui, &[(button_parts::BUTTON, parts::TOASTUNDO)], |ui| undo_btn.ui(ui, look)).is_empty();
                            }
                            let mut close_btn = Button::new(ButtonProps {
                                variant: "ghost".into(),
                                tone: "muted".into(),
                                hover: "wash".into(),
                                size: "sm".into(),
                                shape: "icon".into(),
                                width: "content".into(),
                                icon: "close".into(),
                                iconSize: 16,
                                ariaLabel: "Dismiss".into(),
                                ..Default::default()
                            });
                            close = !as_part(ui, &[(button_parts::BUTTON, parts::TOASTCLOSE)], |ui| close_btn.ui(ui, look)).is_empty();
                        });
                    })
                    .response
                    .rect;
                let node = ui.interact(r, ui.scope_id().with("toast"), Sense::hover());
                node.widget_info(|| WidgetInfo::labeled(Role::Status, true, &message));
                part(ui, &node, parts::TOAST);
            });

        if undo {
            app.undo();
            self.toast_key = None;
        } else if close {
            app.toast = None;
            self.toast_key = None;
        }
    }
}

/// The screen's mount in a test: the app on the props' data set, with the sheet the props name.
struct Mount {
    app: App,
    screen: Sheets,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: SheetsProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    // The demo account's sheets work in the Home project, as the web harness has them.
    let home = app.data.project_named("Home").map(|p| p.id.clone());
    app.open_named(Some("Home"), None);
    let home_ref = home.as_deref();
    let sheet = match p.sheet.as_str() {
        "newList" => Sheet::NewList,
        "newProject" => Sheet::NewProject,
        "renameList" => Sheet::RenameList(list_named(&app, home_ref, &p.list)?),
        "renameProject" => Sheet::RenameProject(project_named(&app, &p.project)?),
        "deleteList" => Sheet::DeleteList(list_named(&app, home_ref, &p.list)?),
        "deleteProject" => Sheet::DeleteProject(project_named(&app, &p.project)?),
        "deleteConfirm" => {
            // The task is selected in its list behind the sheet, as the reference shows it.
            let errands = list_named(&app, home_ref, "Errands")?;
            let task = app
                .data
                .tasks_of(&errands)
                .into_iter()
                .find(|t| t.title == p.task)
                .map(|t| t.id.clone())
                .ok_or_else(|| format!("no task named \"{}\"", p.task))?;
            app.ui.set_project(home.clone());
            app.ui.open_list(Some(errands), Some(0));
            app.ui.selection = vec![task.clone()];
            app.ui.preview = Some(task.clone());
            Sheet::DeleteTask(task)
        }
        other => {
            return Err(format!(
                "sheet \"{other}\" is not one of newList, newProject, renameList, renameProject, deleteList, deleteProject, deleteConfirm"
            ))
        }
    };
    app.sheet = Some(sheet);
    Ok(Box::new(Mount { app, screen: Sheets::new() }))
}

// ---- what each sheet says and does ----------------------------------------------------------

fn config(app: &App, sheet: &Sheet) -> Config {
    let undo_key = if cfg!(target_os = "macos") { "⌘Z" } else { "Ctrl Z" };
    match sheet {
        Sheet::NewList => Config {
            glyph: "list_alt_add",
            danger: false,
            title: "New list".into(),
            detail: None,
            accept: "Create",
            instead: true,
            prompt: Some(Prompt { placeholder: "Name this list", roll: Some(Roll::List) }),
        },
        Sheet::NewProject => Config {
            glyph: "create_new_folder",
            danger: false,
            title: "New project".into(),
            detail: None,
            accept: "Create",
            instead: false,
            prompt: Some(Prompt { placeholder: "Name this project", roll: Some(Roll::Project) }),
        },
        Sheet::RenameList(_) => Config {
            glyph: "edit",
            danger: false,
            title: "Rename list".into(),
            detail: None,
            accept: "Rename",
            instead: false,
            prompt: Some(Prompt { placeholder: "", roll: Some(Roll::List) }),
        },
        Sheet::RenameProject(_) => Config {
            glyph: "edit",
            danger: false,
            title: "Rename project".into(),
            detail: None,
            accept: "Rename",
            instead: false,
            prompt: Some(Prompt { placeholder: "", roll: None }),
        },
        Sheet::DeleteList(id) => {
            let n = app.data.tasks_of(id).len();
            Config {
                glyph: "delete",
                danger: true,
                title: format!("Delete “{}”?", list_name(app, id)),
                detail: Some(if n == 0 { "The list is empty. No undo.".to_owned() } else { format!("Its {} {} with it. No undo.", plural(n, "task", "tasks"), goes(n)) }),
                accept: "Delete list",
                instead: false,
                prompt: None,
            }
        }
        Sheet::DeleteProject(id) => {
            let lists = app.data.lists_of(id);
            let n_lists = lists.len();
            let n_tasks: usize = lists.iter().map(|l| app.data.tasks_of(&l.id).len()).sum();
            Config {
                glyph: "delete",
                danger: true,
                title: format!("Delete “{}”?", project_name(app, id)),
                detail: Some(format!("Its {} and {} go with it. No undo.", plural(n_lists, "list", "lists"), plural(n_tasks, "task", "tasks"))),
                accept: "Delete project",
                instead: false,
                prompt: None,
            }
        }
        Sheet::DeleteTask(id) => {
            let title = app.data.task(id).map(|t| t.title.clone()).unwrap_or_default();
            // A parent goes with its subtasks: the copy counts them.
            let kids = app.data.children_of(id).len();
            let what = if kids == 0 { format!("“{title}”") } else { format!("“{title}” and its {}", plural(kids, "subtask", "subtasks")) };
            Config {
                glyph: "delete",
                danger: true,
                title: "Move to Trash?".into(),
                detail: Some(format!("{what} — {undo_key} puts it back.")),
                accept: "Move to Trash",
                instead: false,
                prompt: None,
            }
        }
        Sheet::EmptyTrash => {
            let n = app.data.trash.len();
            Config {
                glyph: "delete",
                danger: true,
                title: "Empty Trash?".into(),
                detail: Some(format!("{} deleted for good. No undo.", plural(n, "task", "tasks"))),
                accept: "Empty Trash",
                instead: false,
                prompt: None,
            }
        }
        Sheet::PurgeTasks(ids) => {
            let n = ids.len();
            Config {
                glyph: "delete",
                danger: true,
                title: "Delete permanently?".into(),
                detail: Some(format!("{} deleted for good. No undo.", plural(n, "task", "tasks"))),
                accept: "Delete permanently",
                instead: false,
                prompt: None,
            }
        }
    }
}

/// The name a prompt starts with: a rolled name for the new sheets, the current name for the renames.
fn prompt_initial(app: &App, sheet: &Sheet) -> Option<String> {
    match sheet {
        Sheet::NewList => Some(roll_name(Roll::List)),
        Sheet::NewProject => Some(roll_name(Roll::Project)),
        Sheet::RenameList(id) => Some(list_name(app, id)),
        Sheet::RenameProject(id) => Some(project_name(app, id)),
        _ => None,
    }
}

/// Does the accept: the write the sheet is for. A refused save comes back as an error.
fn commit(app: &mut App, sheet: &Sheet, name: &str) -> anyhow::Result<()> {
    match sheet {
        Sheet::NewList => {
            let project = app.ui.project.clone().ok_or_else(|| anyhow::anyhow!("no project is open"))?;
            app.data.create_list(&project, name)?;
        }
        Sheet::NewProject => {
            app.data.create_project(name)?;
        }
        Sheet::RenameList(id) => app.data.rename_list(id, name)?,
        Sheet::RenameProject(id) => app.data.rename_project(id, name)?,
        Sheet::DeleteList(id) => {
            app.data.delete_list(id)?;
            for pane in app.ui.panes.iter_mut() {
                if pane.as_deref() == Some(id.as_str()) {
                    *pane = None;
                }
            }
            let data = &app.data;
            app.ui.selection.retain(|s| data.task(s).is_some());
        }
        Sheet::DeleteProject(id) => {
            app.data.delete_project(id)?;
            if app.ui.project.as_deref() == Some(id.as_str()) {
                let first = app.data.projects.first().map(|p| p.id.clone());
                let list = first.as_deref().and_then(|p| app.data.lists_of(p).first().map(|l| l.id.clone()));
                app.ui.set_project(first);
                app.ui.open_list(list, Some(0));
            }
        }
        Sheet::DeleteTask(id) => app.trash(&[id.clone()]),
        Sheet::EmptyTrash => app.data.empty_trash()?,
        Sheet::PurgeTasks(ids) => app.data.purge_tasks(ids)?,
    }
    Ok(())
}

fn list_named(app: &App, project: Option<&str>, name: &str) -> Result<String, String> {
    app.data.list_named(project, name).map(|l| l.id.clone()).ok_or_else(|| format!("no list named \"{name}\""))
}

fn project_named(app: &App, name: &str) -> Result<String, String> {
    app.data.project_named(name).map(|p| p.id.clone()).ok_or_else(|| format!("no project named \"{name}\""))
}

fn list_name(app: &App, id: &str) -> String {
    app.data.list(id).map(|l| l.name.clone()).unwrap_or_default()
}

fn project_name(app: &App, id: &str) -> String {
    app.data.project(id).map(|p| p.name.clone()).unwrap_or_default()
}

fn answered(action: &str, text: &str, id: &str) -> SheetsEvent {
    SheetsEvent::Answered { action: action.into(), text: text.into(), id: id.into() }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

fn goes(n: usize) -> &'static str {
    if n == 1 {
        "goes"
    } else {
        "go"
    }
}

// ---- drawing helpers --------------------------------------------------------------------------

fn input_id() -> Id {
    // The id the input widget gives its field.
    Id::unique("ui.input.field")
}


/// A name field: bare, selected on focus, focused when it appears.
fn make_input(value: &str, title: &str, placeholder: &str) -> Input {
    Input::new(InputProps {
        value: value.into(),
        placeholder: placeholder.into(),
        ariaLabel: title.into(),
        size: "md".into(),
        mode: "bare".into(),
        autoFocus: true,
        selectOnFocus: true,
        spellCheck: false,
        ..Default::default()
    })
}

/// The 1 px edge the reference's sheet buttons have (the flat lg buttons read as unbordered, #12360).
fn outline(ui: &egui::Ui, look: Look, r: Rect) {
    ui.painter().rect_stroke(r, CornerRadius::same(4), Stroke::new(1.0, look.t.edge_2), StrokeKind::Inside);
}

/// The width of an lg footer button's content: its label and the 10px sides.
fn lg_width(ui: &egui::Ui, label: &str) -> f32 {
    ui.painter().layout_no_wrap(label.to_owned(), FontId::new(14.0, weight(500)), Color32::WHITE).size().x + 2.0 * BTN_SIDE
}

/// Draws a widget in a rect, laid out from its left edge. The salt fixes the child Ui's id.
fn at<R>(ui: &mut egui::Ui, salt: &str, r: Rect, f: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.scope_builder(UiBuilder::new().id_salt(salt).max_rect(r).layout(Layout::left_to_right(Align::Center)), f).inner
}

// ---- rolled names -----------------------------------------------------------------------------

fn roll_name(kind: Roll) -> String {
    match kind {
        Roll::List => format!("{} {}", pick(&LIST_FIRST), pick(&LIST_SECOND)),
        Roll::Project => pick(&PROJECT_NAMES).to_owned(),
    }
}

fn pick(items: &[&'static str]) -> &'static str {
    items[(next_random() % items.len() as u64) as usize]
}

static STATE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Starts the dice from a fixed seed: each test mount rolls the same names, so a press and its keyboard
/// twin open the same sheet.
pub fn reset_roll() {
    STATE.store(0x9E37_79B9_7F4A_7C15, std::sync::atomic::Ordering::Relaxed);
}

/// A xorshift generator, seeded from the clock: the dice need no more than that.
fn next_random() -> u64 {
    use std::sync::atomic::Ordering;
    let mut s = STATE.load(Ordering::Relaxed);
    if s == 0 {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0);
        s = nanos | 1;
    }
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    STATE.store(s, Ordering::Relaxed);
    s
}
