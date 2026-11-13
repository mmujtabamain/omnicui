//! The registry: for each contract id, the app's surface built from the test's props, the way the
//! app itself puts it in that state (SPEC-PRODUCT.md, "What every target does"). omnicui sends no
//! defaults, so each contract's defaults are here too.

use std::rc::Rc;

use gpui::{App, AppContext, Context, Entity, Window};
use serde_json::{Map, Value};
use taskscape_gpui::{capture::CaptureBar, settings::SettingsView, workspace::SheetKind, Db, Surface, Workspace};

pub enum Root {
    Ws(Entity<Workspace>),
    Settings(Entity<SettingsView>),
    Capture(Entity<CaptureBar>),
}

/// The contracts' defaults (`props: { x: { default } }`), per component.
pub fn with_defaults(component: &str, props: &Value) -> Map<String, Value> {
    let d: &[(&str, Value)] = match component {
        "taskscape.pane" => &[("seed", s("standard")), ("list", s("launch")), ("showCompleted", Value::Bool(true)), ("multi", s(""))],
        "taskscape.inspector" => &[("seed", s("standard")), ("selected", s("")), ("draft", s("")), ("multi", s(""))],
        "taskscape.titlebar" => &[("seed", s("standard")), ("list", s("launch")), ("split", s("")), ("preview", Value::Bool(true))],
        "taskscape.sheet" => &[("seed", s("standard")), ("kind", s("deleteTask")), ("list", s("launch")), ("task", s("brief")), ("project", s("atlas"))],
        "taskscape.palette" => &[("seed", s("standard")), ("selected", s(""))],
        "taskscape.filter" => &[("seed", s("standard")), ("list", s("launch"))],
        "taskscape.toast" => &[("seed", s("standard")), ("trashed", s("banners"))],
        _ => &[("seed", s("standard"))],
    };
    let mut m: Map<String, Value> = d.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
    if let Some(given) = props.as_object() {
        for (k, v) in given {
            m.insert(k.clone(), v.clone());
        }
    }
    m
}

fn s(v: &str) -> Value {
    Value::String(v.into())
}

pub fn surface(component: &str) -> Option<Surface> {
    Some(match component {
        "taskscape.window" => Surface::Window,
        "taskscape.pane" => Surface::Pane,
        "taskscape.inspector" => Surface::Inspector,
        "taskscape.titlebar" => Surface::TitleBar,
        "taskscape.sheet" => Surface::Sheet,
        "taskscape.palette" => Surface::Palette,
        "taskscape.filter" => Surface::Filter,
        "taskscape.trash" => Surface::Trash,
        "taskscape.toast" => Surface::Toast,
        _ => return None,
    })
}

pub const COMPONENTS: &[&str] = &[
    "taskscape.window",
    "taskscape.pane",
    "taskscape.inspector",
    "taskscape.titlebar",
    "taskscape.sheet",
    "taskscape.palette",
    "taskscape.filter",
    "taskscape.trash",
    "taskscape.toast",
    "taskscape.settings",
    "taskscape.capture",
];

/// Builds the surface in a new window; `id` turns a fixture key into the store's id.
pub fn build(component: &str, props: &Map<String, Value>, db: Rc<Db>, dark: bool, id: &dyn Fn(&str) -> Option<String>, window: &mut Window, cx: &mut App) -> Result<Root, String> {
    let str_prop = |k: &str| props.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let key_id = |k: &str| -> Result<Option<String>, String> {
        let v = str_prop(k);
        if v.is_empty() {
            return Ok(None);
        }
        id(&v).map(Some).ok_or_else(|| format!("no `{v}` in the fixture (prop {k})"))
    };
    match component {
        "taskscape.settings" => {
            let v = cx.new(|cx| SettingsView::new(db, None, dark, window, cx));
            let f = v.read(cx).focus.clone();
            window.focus(&f);
            Ok(Root::Settings(v))
        }
        "taskscape.capture" => Ok(Root::Capture(cx.new(|cx| CaptureBar::new(db, dark, window, cx)))),
        other => {
            let surface = surface(other).ok_or_else(|| format!("no component {other}"))?;
            let list = key_id("list")?;
            let split = key_id("split")?;
            let selected = key_id("selected")?;
            let draft = key_id("draft")?;
            let multi: Vec<String> = str_prop("multi").split(',').map(str::trim).filter(|s| !s.is_empty()).map(|k| id(k).ok_or_else(|| format!("no `{k}` in the fixture (prop multi)"))).collect::<Result<_, _>>()?;
            let kind = str_prop("kind");
            let task = key_id("task")?;
            let project = key_id("project")?;
            let trashed = key_id("trashed")?;
            let preview = props.get("preview").and_then(|v| v.as_bool());
            let ws = cx.new(|cx: &mut Context<Workspace>| Workspace::new(db, surface, dark, window, cx));
            let mut err = None;
            ws.update(cx, |ws, cx| {
                let f = ws.root_focus.clone();
                window.focus(&f);
                if surface != Surface::Window {
                    if let Some(l) = &list {
                        ws.show_list(l, window, cx);
                    } else if props.contains_key("list") && surface == Surface::TitleBar {
                        ws.panes[0].list = None;
                    }
                }
                if let Some(s) = &split {
                    ws.set_split(Some(s), window, cx);
                }
                if let Some(p) = preview {
                    ws.set_preview(p, cx);
                }
                if let Some(s) = &selected {
                    ws.focus_task(s, window, cx);
                    let f = ws.panes[ws.focused_pane].focus.clone();
                    window.focus(&f);
                }
                if let Some(d) = &draft {
                    ws.open_draft(d, window, cx);
                }
                if !multi.is_empty() {
                    ws.select(multi.clone(), cx);
                }
                match surface {
                    Surface::Palette => ws.open_palette(window, cx),
                    Surface::Toast => {
                        if let Some(t) = &trashed {
                            ws.trash(&[t.clone()], cx);
                        }
                    }
                    Surface::Sheet => {
                        let list_id = list.clone().unwrap_or_default();
                        let k = match kind.as_str() {
                            "deleteTask" => Some(SheetKind::DeleteTasks(vec![task.clone().unwrap_or_default()])),
                            "deleteList" => Some(SheetKind::DeleteList(list_id)),
                            "newList" => Some(SheetKind::NewList),
                            "renameList" => Some(SheetKind::RenameList(list_id)),
                            "newProject" => Some(SheetKind::NewProject),
                            "renameProject" => Some(SheetKind::RenameProject(project.clone().unwrap_or_default())),
                            "deleteProject" => Some(SheetKind::DeleteProject(project.clone().unwrap_or_default())),
                            _ => None,
                        };
                        match k {
                            Some(k) => ws.open_sheet(k, window, cx),
                            None => err = Some(format!("no sheet {kind}")),
                        }
                    }
                    _ => {}
                }
            });
            match err {
                Some(e) => Err(e),
                None => Ok(Root::Ws(ws)),
            }
        }
    }
}
