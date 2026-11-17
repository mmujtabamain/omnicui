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
