//! The demo data loads into a fresh store the way the screen contracts expect it.
use taskscape_storage::{demo, Store};

#[tokio::test]
async fn standard_demo_data_matches_the_web() {
    let dir = std::env::temp_dir().join(format!("taskscape-demo-{}", std::process::id()));
    std::env::set_var("TASKSCAPE_DATA_DIR", &dir);
    let store = Store::open().await.unwrap();
    demo::load(&store, "standard").await.unwrap();

    let projects: Vec<String> = store.list_projects().await.unwrap().into_iter().map(|p| p.name).collect();
    assert_eq!(projects.len(), 2, "{projects:?}");
    let lists = store.list_lists().await.unwrap();
    let errands = lists.iter().find(|l| l.name == "Errands").expect("Errands");
    let tasks = store.list_tasks(&errands.id).await.unwrap();
    let titles: Vec<&str> = tasks.iter().map(|t| t.title.as_str()).collect();
    assert_eq!(titles.len(), 8, "Errands has 8 live tasks: {titles:?}");
    assert_eq!(tasks.iter().filter(|t| t.done).count(), 2, "2/8 done");
    assert_eq!(store.list_trashed().await.unwrap().len(), 2, "two tasks in the Trash");

    // Loading again replaces everything rather than adding to it.
    demo::load(&store, "empty").await.unwrap();
    assert_eq!(store.list_lists().await.unwrap().len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}
