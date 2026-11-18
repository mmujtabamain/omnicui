//! Replays the reference viewer's states through Taskscape's egui screens.
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Rect, pos2, vec2};
use egui_kittest::{Harness, kittest::NodeT};
use serde_json::{Value, json};
use std::{error::Error, fs, path::Path};
use taskscape_app::{
    App,
    app::Sheet,
    data::Data,
    screens::{capturebar::CaptureBar, mainwindow::MainWindow, settingswindow::SettingsWindow},
    state::Aside,
};
use taskscape_ui::Look;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const CMD: Modifiers = Modifiers {
    command: true,
    mac_cmd: true,
    ..Modifiers::NONE
};
const SHIFT_CMD: Modifiers = Modifiers { shift: true, ..CMD };
const BIKE: &str = "Book bike service";
const KEYS: &str = "Cut a spare key";
const GROCERIES: &str = "Buy groceries for the week";
const READING: &str = "Reading";
const PRESCRIPTION: &str = "Pick up prescription";

enum Screen {
    Main(Box<MainWindow>),
    Settings(Box<SettingsWindow>),
    Capture(Box<CaptureBar>),
}

struct State {
    app: App,
    screen: Screen,
}
type Viewer = Harness<'static, State>;

fn locate(h: &Viewer, selector: &str) -> Result<Rect> {
    let root = h.root();
    root.children_recursive()
        .filter_map(|node| {
            let data = node.accesskit_node();
            let rect = data.bounding_box().map(|b| {
                Rect::from_min_max(
                    pos2(b.x0 as f32, b.y0 as f32),
                    pos2(b.x1 as f32, b.y1 as f32),
                )
            })?;
            let (role, label, value) = (data.role(), data.label(), data.value());
            let score = if data.author_id() == Some(selector) {
                4
            } else if label.as_deref() == Some(selector) || value.as_deref() == Some(selector) {
                if role == egui::accesskit::Role::Button {
                    3
                } else {
                    2
                }
            } else if selector.starts_with('~') && label.is_some_and(|s| s.contains(&selector[1..]))
            {
                1
            } else {
                0
            };
            (score > 0 && rect.is_positive()).then_some((score, rect))
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, rect)| rect)
        .ok_or_else(|| format!("cannot find {selector:?}").into())
}

fn click_at(h: &mut Viewer, pos: Pos2, button: PointerButton, modifiers: Modifiers) {
    h.hover_at(pos);
    h.event(Event::ModifiersChanged(modifiers));
    h.run_steps(3);
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos,
            button,
            pressed,
            modifiers,
        });
        h.step();
    }
    h.run_steps(12);
}

fn click(h: &mut Viewer, selector: &str, modifiers: Modifiers) -> Result<()> {
    click_at(
        h,
        locate(h, selector)?.center(),
        PointerButton::Primary,
        modifiers,
    );
    Ok(())
}

fn preview(h: &mut Viewer, title: &str) -> Result<()> {
    let app = &mut h.state_mut().app;
    let task = app
        .data
        .tasks
        .iter()
        .find(|task| task.title == title)
        .ok_or("missing task")?;
    app.ui.preview = Some(task.id.clone());
    h.run_steps(6);
    Ok(())
}

fn right_row(h: &mut Viewer, title: &str) -> Result<()> {
    let row = locate(h, title)?;
    click_at(
        h,
        pos2(row.left() + 55.0, row.center().y),
        PointerButton::Secondary,
        Modifiers::NONE,
    );
    Ok(())
}

fn right(h: &mut Viewer, selector: &str) -> Result<()> {
    click_at(
        h,
        locate(h, selector)?.center(),
        PointerButton::Secondary,
        Modifiers::NONE,
    );
    Ok(())
}

fn hover(h: &mut Viewer, selector: &str) -> Result<()> {
    h.hover_at(locate(h, selector)?.center());
    h.run_steps(12);
    Ok(())
}

fn hold(h: &mut Viewer, modifiers: Modifiers) {
    h.event(Event::ModifiersChanged(modifiers));
    h.run_steps(75);
}

fn key(h: &mut Viewer, key: Key, modifiers: Modifiers) {
    h.key_press_modifiers(modifiers, key);
    h.run_steps(15);
}

fn enter_text(h: &mut Viewer, text: &str) {
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.event(Event::Text(text.into()));
    h.run_steps(15);
}

fn scroll(h: &mut Viewer, pos: Pos2, distance: f32) {
    h.hover_at(pos);
    h.run_steps(3);
    h.event(Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: vec2(0.0, -distance),
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(50);
}

fn tab_sheet(h: &mut Viewer, delete: bool) -> Result<()> {
    let app = &mut h.state_mut().app;
    let list = app
        .data
        .lists
        .iter()
        .find(|l| l.name == READING)
        .ok_or("missing Reading list")?;
    app.sheet = Some(if delete {
        Sheet::DeleteList(list.id.clone())
    } else {
        Sheet::RenameList(list.id.clone())
    });
    Ok(())
}

fn open_launch(h: &mut Viewer) {
    h.state_mut()
        .app
        .open_named(Some("Spring launch"), Some("Website"));
    h.run_steps(15);
}

fn run_state(h: &mut Viewer, id: &str) -> Result<()> {
    let none = Modifiers::NONE;
    match id {
        "main-default" | "main-dark" | "narrow-window" | "empty-list" | "no-lists"
        | "long-list" | "settings-general" | "settings-dark" | "capture-bar"
        | "capture-bar-dark" => {}
        "row-hover" => hover(h, BIKE)?,
        "row-selected" | "row-selected-dark" => click(h, BIKE, none)?,
        "row-with-subtasks-selected" => click(h, GROCERIES, none)?,
        "subtask-selected" => click(h, "Sourdough loaf", none)?,
        "done-row-selected" => click(h, "Post the parcel to Mum", none)?,
        "multi-select-cmd" | "multi-select-cmd-two" | "multi-context-menu" => {
            click(
                h,
                BIKE,
                if id == "multi-select-cmd-two" {
                    CMD
                } else {
                    none
                },
            )?;
            click(h, KEYS, CMD)?;
            if id == "multi-context-menu" {
                h.state_mut().app.ui.selection.clear();
                preview(h, KEYS)?;
                right_row(h, KEYS)?;
            }
        }
        "multi-select-shift" => {
            click(h, "Pick up prescription", none)?;
            click(h, KEYS, Modifiers::SHIFT)?;
        }
        "option-held" => hold(h, Modifiers::ALT),
        "option-held-hover" => {
            hover(h, BIKE)?;
            hold(h, Modifiers::ALT);
        }
        "option-held-selected" => {
            click(h, BIKE, none)?;
            hold(h, Modifiers::ALT);
        }
        "cmd-held" => hold(h, CMD),
        "shift-held" => hold(h, Modifiers::SHIFT),
        "row-context-menu" => {
            preview(h, BIKE)?;
            right_row(h, BIKE)?;
        }
        "row-more-menu" => {
            hover(h, BIKE)?;
            preview(h, PRESCRIPTION)?;
            let row = locate(h, PRESCRIPTION)?;
            let at = h
                .root()
                .children_recursive()
                .filter_map(|node| {
                    let data = node.accesskit_node();
                    if data.label().as_deref() != Some("More") {
                        return None;
                    }
                    data.bounding_box()?;
                    Some(node.rect().center())
                })
                .min_by(|a, b| {
                    (a.y - row.center().y)
                        .abs()
                        .total_cmp(&(b.y - row.center().y).abs())
                })
                .ok_or("missing More button")?;
            click_at(h, at, PointerButton::Primary, none);
        }
        "tab-context-menu" => right(h, READING)?,
        "tab-hover" => hover(h, READING)?,
        "project-menu" => click(h, "Home", none)?,
        "project-launch" => open_launch(h),
        "inspector-notes" | "note-editing" | "attachment-lightbox" | "attachment-menu" => {
            open_launch(h);
            click(h, "Rewrite the hero section", none)?;
            match id {
                "note-editing" => click(h, "app.taskinspector.noteBodySecond", none)?,
                "attachment-lightbox" => {
                    click(h, "app.taskinspector.attachmentTileImage", none)?;
                    locate(h, "app.taskinspector.lightbox")?;
                }
                "attachment-menu" => right(h, "app.taskinspector.attachmentTileLink")?,
                _ => {}
            }
        }
        "palette" | "palette-search" | "palette-no-match" => {
            key(h, Key::K, CMD);
            match id {
                "palette-search" => enter_text(h, "new"),
                "palette-no-match" => enter_text(h, "zzzz"),
                _ => {}
            }
        }
        "search-typed" | "search-no-match" => {
            key(h, Key::F, CMD);
            enter_text(h, if id == "search-typed" { "b" } else { "zzzz" });
        }
        "filter-panel" => h.state_mut().app.ui.aside = Aside::Filter,
        "new-task-draft" | "new-task-typed" | "new-task-added" => {
            key(h, Key::N, CMD);
            if id != "new-task-draft" {
                enter_text(h, "Water the plants");
            }
            if id == "new-task-added" {
                key(h, Key::Enter, none);
                if !h
                    .state()
                    .app
                    .data
                    .tasks
                    .iter()
                    .any(|t| t.title == "Water the plants")
                {
                    return Err("new task did not save".into());
                }
            }
        }
        "rename-inline" => {
            click(h, BIKE, none)?;
            key(h, Key::Enter, none);
        }
        "sheet-new-list" => key(h, Key::N, SHIFT_CMD),
        "sheet-new-project" => h.state_mut().app.sheet = Some(Sheet::NewProject),
        "sheet-rename-list" => tab_sheet(h, false)?,
        "sheet-delete-list" => tab_sheet(h, true)?,
        "delete-confirm" | "toast-after-delete" | "after-undo" => {
            click(h, BIKE, none)?;
            key(h, Key::Backspace, CMD);
            if h.state().app.sheet.is_none() {
                return Err("delete did not open a confirmation".into());
            }
            if id != "delete-confirm" {
                click(h, "app.mainwindow.dialogMoveToTrash", none)?;
                if !h
                    .state()
                    .app
                    .data
                    .trash
                    .iter()
                    .any(|task| task.title == BIKE)
                {
                    return Err("task was not moved to Trash".into());
                }
            }
            if id == "after-undo" {
                key(h, Key::Z, CMD);
                if !h
                    .state()
                    .app
                    .data
                    .tasks
                    .iter()
                    .any(|task| task.title == BIKE)
                {
                    return Err("undo did not restore the task".into());
                }
            }
        }
        "option-click-check" | "round-check-click" => {
            let row = locate(h, BIKE)?;
            let modifiers = if id == "option-click-check" {
                Modifiers::ALT
            } else {
                none
            };
            hold(h, modifiers);
            let root = h.root();
            let at = root
                .children_recursive()
                .find_map(|n| {
                    let data = n.accesskit_node();
                    data.bounding_box()?;
                    let rect = n.rect();
                    let label = data.label();
                    (rect.is_positive()
                        && rect.width() <= 40.0
                        && rect.center().x < row.center().x
                        && (rect.center().y - row.center().y).abs() < 12.0
                        && matches!(
                            label.as_deref(),
                            Some("Select" | "Mark done" | "Select task" | "Deselect")
                        ))
                    .then_some(rect.center())
                })
                .unwrap_or(pos2(40.0, row.center().y));
            click_at(h, at, PointerButton::Primary, modifiers);
            if id == "round-check-click" {
                preview(h, BIKE)?;
            }
        }
        "trash" | "trash-selected" => {
            h.state_mut().app.trash_window = true;
            h.run_steps(25);
            if id == "trash-selected" {
                click(h, "~Print the old flyer", none)?;
            }
        }
        "completed-hidden" => h
            .state_mut()
            .app
            .data
            .set_setting("show_completed", "false")?,
        "split-view" => h.state_mut().app.ui.split(true),
        "preview-hidden" => h.state_mut().app.ui.aside = Aside::None,
        "collapsed-all" => click(h, "app.mainwindow.rowGroceriesExpand", none)?,
        "list-switch-3" => key(h, Key::Num3, CMD),
        "long-list-scrolled" => scroll(h, pos2(400.0, 400.0), 900.0),
        "settings-capture"
        | "settings-shortcuts"
        | "settings-about"
        | "settings-shortcuts-scrolled"
        | "settings-shortcut-recording" => {
            let pane = match id {
                "settings-capture" => "capture",
                "settings-about" => "about",
                _ => "shortcuts",
            }
            .to_owned();
            h.state_mut().app.settings_window = Some(pane);
            h.run_steps(20);
            if id == "settings-shortcuts-scrolled" {
                scroll(h, pos2(700.0, 400.0), 1200.0);
            }
            if id == "settings-shortcut-recording" {
                click(h, "app.settingsshortcuts.keycapNewTask", none)?;
            }
        }
        "settings-search" => enter_text(h, "motion"),
        "capture-bar-typed" | "capture-bar-notes" => {
            enter_text(h, "Call the plumber");
            if id == "capture-bar-notes" {
                key(h, Key::Tab, none);
            }
        }
        _ => return Err(format!("no egui scenario for {id}").into()),
    }
    h.run_steps(30);
    Ok(())
}

fn write_tree(h: &Viewer, dir: &Path) -> Result<()> {
    fn walk(
        node: egui_kittest::Node<'_>,
        depth: usize,
        lines: &mut String,
        elements: &mut Vec<Value>,
    ) {
        use std::fmt::Write;
        let data = node.accesskit_node();
        let role = format!("{:?}", data.role());
        let (part, label, value) = (data.author_id(), data.label(), data.value());
        let rect = data.bounding_box().map(|b| {
            Rect::from_min_max(
                pos2(b.x0 as f32, b.y0 as f32),
                pos2(b.x1 as f32, b.y1 as f32),
            )
        });
        let coordinates = rect.map(|r| [r.min.x, r.min.y, r.width(), r.height()]);
        let _ = writeln!(
            lines,
            "{:depth$}{role} {} {} @{:?}",
            "",
            part.unwrap_or(""),
            label.as_deref().or(value.as_deref()).unwrap_or(""),
            coordinates
        );
        if let Some(coordinates) = coordinates
            .filter(|box4| box4[2] > 0.0 && box4[3] > 0.0 && box4.iter().all(|v| v.is_finite()))
        {
            elements.push(
                json!({"role": role.to_lowercase(), "part": part, "label": label,
                "text": value, "box": coordinates.map(|x| x.round() as i32)}),
            );
        }
        for child in node.children() {
            walk(child, depth + 2, lines, elements);
        }
    }
    let mut lines = String::new();
    let mut elements = Vec::new();
    walk(h.root(), 0, &mut lines, &mut elements);
    fs::write(dir.join("tree.txt"), lines)?;
    fs::write(dir.join("elements.json"), serde_json::to_vec(&elements)?)?;
    Ok(())
}

fn capture(meta: &Value, dir: &Path) -> Result<()> {
    let id = meta["id"].as_str().ok_or("missing id")?;
    let dark = meta["theme"] == "dark";
    let look = Look::of(if dark { "dark" } else { "light" });
    let size = vec2(
        meta["size"][0].as_f64().ok_or("missing width")? as f32,
        meta["size"][1].as_f64().ok_or("missing height")? as f32,
    );
    let clock = meta["epoch"].as_i64().ok_or("missing reference clock")?;
    // The recorder owns the process and mounts one state at a time. Store workers from
    // the preceding state have been dropped before changing the isolated data directory.
    unsafe {
        std::env::set_var(
            "TASKSCAPE_DATA_DIR",
            meta["data"].as_str().ok_or("missing data")?,
        )
    };
    let mut app = App::new(Data::open()?);
    app.native_window = true;
    let screen = match meta["window"].as_str() {
        Some("settings") => {
            let mut settings = SettingsWindow::new();
            settings.general_chrome = (taskscape_app::chrome::lead(true), 0.0);
            Screen::Settings(Box::new(settings))
        }
        Some("tray") => Screen::Capture(Box::new(CaptureBar::new())),
        _ => Screen::Main(Box::new(MainWindow::new())),
    };
    let symbols = fs::read("/System/Library/Fonts/Apple Symbols.ttf")?;
    let mut h = Harness::builder()
        .with_size(size)
        .with_pixels_per_point(1.0)
        .with_step_dt(1.0 / 60.0)
        .with_render_cursor(false)
        .with_accessibility_check(false)
        .with_theme(if dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        })
        .with_context_setup(move |ctx| {
            taskscape_ui::fonts(ctx);
            let families = [400, 500, 600, 700].map(taskscape_ui::weight);
            ctx.add_font(egui::FontInsert::new(
                "Apple Symbols",
                egui::FontData::from_owned(symbols),
                families
                    .into_iter()
                    .map(|family| egui::InsertFontFamily {
                        family,
                        priority: egui::FontPriority::Lowest,
                    })
                    .collect(),
            ));
            taskscape_app::time::set_capture_time(ctx, clock);
        })
        .build_ui_state(
            move |ui, s: &mut State| {
                let full = ui.ctx().content_rect();
                ui.scope_builder(egui::UiBuilder::new().max_rect(full), |ui| {
                    ui.set_clip_rect(full);
                    let background = if matches!(s.screen, Screen::Capture(_)) {
                        egui::Color32::WHITE
                    } else {
                        look.t.surface_1
                    };
                    ui.painter().rect_filled(full, 0.0, background);
                    match &mut s.screen {
                        Screen::Main(window) => {
                            window.show(&mut s.app, ui, look);
                        }
                        Screen::Settings(window) => {
                            window.show(&mut s.app, ui, look);
                        }
                        Screen::Capture(bar) => {
                            bar.show(&mut s.app, ui, look);
                        }
                    }
                });
            },
            State { app, screen },
        );
    h.run_steps(24);
    let result = run_state(&mut h, id);
    fs::create_dir_all(dir)?;
    h.render()?.save(dir.join("shot.png"))?;
    write_tree(&h, dir)?;
    let mut meta = meta.clone();
    meta.as_object_mut().unwrap().remove("data");
    meta.as_object_mut().unwrap().remove("commands");
    meta["target"] = json!("egui");
    meta["scale"] = json!(1);
    meta["problems"] = match &result {
        Err(e) => json!([e.to_string()]),
        Ok(()) => json!([]),
    };
    fs::write(dir.join("meta.json"), serde_json::to_vec_pretty(&meta)?)?;
    result
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let request = args.next().ok_or("expected manifest")?;
    let output = args.next().ok_or("expected output")?;
    let states: Vec<Value> = serde_json::from_slice(&fs::read(request)?)?;
    let mut clean = 0;
    for state in &states {
        let id = state["id"].as_str().ok_or("missing state id")?;
        match std::panic::catch_unwind(|| capture(state, &Path::new(&output).join(id))) {
            Ok(Ok(())) => {
                clean += 1;
                println!("ok {id}");
            }
            Ok(Err(e)) => println!("!! {id}: {e}"),
            Err(_) => println!("!! {id}: capture panicked"),
        }
    }
    println!("{clean}/{} clean", states.len());
    if clean == states.len() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}
