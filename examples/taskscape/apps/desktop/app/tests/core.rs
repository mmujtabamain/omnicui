//! The app core on the demo data: where it opens, Move to Trash with Undo, and a write that shows at once.
use taskscape_app::App;

#[test]
fn opens_on_the_last_list_and_trash_undo_round_trips() {
    let mut app = App::demo("standard").unwrap();
    let errands = app.data.list_named(None, "Errands").unwrap().id.clone();
    assert_eq!(app.active_list().as_deref(), Some(errands.as_str()), "opens on Home / Errands");
    assert_eq!(app.data.tasks_of(&errands).len(), 8);

    let key = app.data.task_named("Cut a spare key").unwrap().id.clone();
    app.trash(&[key.clone()]);
    assert!(app.data.task(&key).is_none(), "gone from the list at once");
    assert_eq!(app.toast.as_ref().map(|t| t.message.as_str()), Some("Moved to Trash"));
    assert_eq!(app.data.trash.first().map(|t| t.title.as_str()), Some("Cut a spare key"), "newest first in the Trash");

    app.undo();
    assert!(app.toast.is_none());
    assert!(app.data.task(&key).is_some(), "Undo puts it back");
}

#[test]
fn a_rename_and_a_done_show_and_persist() {
    let mut app = App::demo("standard").unwrap();
    let bike = app.data.task_named("Book bike service").unwrap().id.clone();
    app.data.update_task(&bike, Some("Book the bike service"), None, Some(true)).unwrap();
    app.data.reload().unwrap();
    let t = app.data.task(&bike).unwrap();
    assert_eq!(t.title, "Book the bike service");
    assert!(t.done);
}

#[test]
fn open_named_moves_to_another_project() {
    let mut app = App::demo("standard").unwrap();
    app.open_named(Some("Spring launch"), Some("Press kit"));
    let press = app.data.list_named(app.ui.project.as_deref(), "Press kit").unwrap().id.clone();
    assert_eq!(app.active_list(), Some(press));
}

/// The settings file is shared with the reference app, which writes Show completed and Reduced motion as
/// `1` and `0`: each app must read what the other wrote (review #12354).
#[test]
fn settings_read_and_write_in_the_reference_format() {
    let mut app = App::demo("standard").unwrap();
    let file = taskscape_storage::paths::root_dir().join("settings.json");
    std::fs::write(&file, r#"{"show_completed":"0","reduced_motion":"1","theme":"dark"}"#).unwrap();
    app.data.reload().unwrap();
    assert!(!app.data.settings.show_completed, "the reference's 0 hides completed tasks");
    assert_eq!(app.data.settings.reduced_motion, "on", "the reference's 1 is reduced motion on");
    assert_eq!(app.data.settings.theme, "dark");

    app.data.set_setting("show_completed", "true").unwrap();
    app.data.set_setting("reduced_motion", "system").unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(saved["show_completed"], "1", "written as the reference writes it");
    assert_eq!(saved["reduced_motion"], "0");
}

/// A list exported in the reference's format comes back whole: its tasks, subtasks, done state and notes
/// (review #12350: export and import were never built on the desktop).
#[test]
fn a_list_exports_and_imports_in_the_reference_format() {
    let mut app = App::demo("standard").unwrap();
    let errands = app.data.list_named(None, "Errands").unwrap().id.clone();
    let json = app.data.export_list(&errands).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(doc["taskscape_export"], 1);
    assert_eq!(doc["list"]["name"], "Errands");
    let home = app.data.project_named("Home").unwrap().id.clone();
    let copy = app.data.import_list(&home, &json, Some("Errands copy")).unwrap();
    let before = app.data.tasks_of(&errands);
    let after = app.data.tasks_of(&copy);
    assert_eq!(after.len(), before.len(), "every task, subtasks included");
    assert_eq!(after.iter().filter(|t| t.done).count(), before.iter().filter(|t| t.done).count());
    let groceries = after.iter().find(|t| t.title == "Buy groceries for the week").unwrap();
    assert_eq!(app.data.children_of(&groceries.id).len(), 3);
}
