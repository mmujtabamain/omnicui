//! The app's data: the reference's storage (SQLite through SeaORM), kept in memory and written
//! through. Every write changes the in-memory copy first, so the screen shows it at once (as the
//! web's instant feedback does), then saves; a refused save reloads from the database and returns
//! the error, which the caller shows.
use anyhow::Result;
use taskscape_storage::{demo, Attachment, List, Note, Project, Store, Task};

pub use taskscape_storage::LinkType;

pub type Id = String;

/// The account-free settings the desktop keeps locally (decision 17).
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// "system", "light" or "dark".
    pub theme: String,
    /// "system", "on" or "off" (decision 16).
    pub reduced_motion: String,
    pub show_completed: bool,
}

pub struct Data {
    rt: tokio::runtime::Runtime,
    store: Store,
    pub projects: Vec<Project>,
    pub lists: Vec<List>,
    /// Every live task, in list order (sort_order, then creation).
    pub tasks: Vec<Task>,
    /// Trashed tasks, newest first.
    pub trash: Vec<Task>,
    pub settings: Settings,
}

impl Data {
    /// Opens the store in TASKSCAPE_DATA_DIR (or ~/.taskscape) and loads everything.
    pub fn open() -> Result<Self> {
        let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build()?;
        let store = rt.block_on(Store::open())?;
        let mut d = Self { rt, store, projects: vec![], lists: vec![], tasks: vec![], trash: vec![], settings: Settings { theme: "system".into(), reduced_motion: "system".into(), show_completed: true } };
        d.reload()?;
        Ok(d)
    }

    /// Opens a store in a fresh folder of its own, loaded with a demo data set (standard, empty,
    /// nolists, long): what each screen test starts from.
    pub fn demo(dataset: &str) -> Result<Self> {
        let dir = std::env::temp_dir().join(format!("taskscape-{}-{}", std::process::id(), NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
        let _ = std::fs::remove_dir_all(&dir);
        // SAFETY: the target mounts one screen at a time on one thread; nothing reads the variable concurrently.
        unsafe { std::env::set_var("TASKSCAPE_DATA_DIR", &dir) };
        let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build()?;
        let store = rt.block_on(Store::open())?;
        rt.block_on(demo::load(&store, dataset))?;
        let mut d = Self { rt, store, projects: vec![], lists: vec![], tasks: vec![], trash: vec![], settings: Settings { theme: "system".into(), reduced_motion: "system".into(), show_completed: true } };
        d.reload()?;
        Ok(d)
    }

    /// Reads everything again from the database.
    pub fn reload(&mut self) -> Result<()> {
        let s = &self.store;
        let (projects, lists, tasks, mut trash) = self.rt.block_on(async { Ok::<_, anyhow::Error>((s.list_projects().await?, s.list_lists().await?, s.all_tasks().await?, s.list_trashed().await?)) })?;
        trash.sort_by_key(|t| std::cmp::Reverse(t.deleted_at.unwrap_or(0)));
        self.projects = projects;
        self.lists = lists;
        self.tasks = tasks;
        self.trash = trash;
        let get = |k: &str| self.rt.block_on(s.get_setting(k)).ok().flatten();
        self.settings = Settings {
            theme: get("theme").unwrap_or_else(|| "system".into()),
            reduced_motion: get("reduced_motion").unwrap_or_else(|| "system".into()),
            show_completed: get("show_completed").map(|v| v != "false").unwrap_or(true),
        };
        Ok(())
    }

    /// Runs one write against the store; on failure the in-memory copy is read back from the database.
    fn save<T>(&mut self, f: impl AsyncFnOnce(&Store) -> Result<T>) -> Result<T> {
        let out = self.rt.block_on(f(&self.store));
        let _ = self.reload();
        out
    }

    // ---- reads ---------------------------------------------------------------------------

    pub fn project(&self, id: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == id)
    }
    pub fn project_named(&self, name: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.name == name)
    }
    pub fn list(&self, id: &str) -> Option<&List> {
        self.lists.iter().find(|l| l.id == id)
    }
    /// A list by name, in a project (or anywhere when the project is None).
    pub fn list_named(&self, project: Option<&str>, name: &str) -> Option<&List> {
        self.lists.iter().find(|l| l.name == name && project.is_none_or(|p| l.project_id == p))
    }
    pub fn lists_of(&self, project: &str) -> Vec<&List> {
        self.lists.iter().filter(|l| l.project_id == project).collect()
    }
    pub fn task(&self, id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }
    pub fn task_named(&self, title: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.title == title)
    }
    /// The live tasks of a list, in order.
    pub fn tasks_of(&self, list: &str) -> Vec<&Task> {
        self.tasks.iter().filter(|t| t.list_id == list).collect()
    }
    pub fn children_of(&self, parent: &str) -> Vec<&Task> {
        self.tasks.iter().filter(|t| t.parent_id.as_deref() == Some(parent)).collect()
    }
    pub fn notes_of(&self, task: &str) -> Vec<Note> {
        self.task(task).map(|t| t.note_items.clone()).unwrap_or_default()
    }
    pub fn attachments_of(&self, task: &str) -> Vec<Attachment> {
        self.task(task).map(|t| t.attachments.clone()).unwrap_or_default()
    }

    // ---- writes (each shows at once, then saves) ---------------------------------------------

    pub fn create_project(&mut self, name: &str) -> Result<Project> {
        let name = name.to_owned();
        self.save(async move |s| s.create_project(&name).await)
    }
    pub fn rename_project(&mut self, id: &str, name: &str) -> Result<()> {
        if let Some(p) = self.projects.iter_mut().find(|p| p.id == id) {
            p.name = name.to_owned();
        }
        let (id, name) = (id.to_owned(), name.to_owned());
        self.save(async move |s| s.rename_project(&id, &name).await)
    }
    pub fn delete_project(&mut self, id: &str) -> Result<()> {
        self.projects.retain(|p| p.id != id);
        let id = id.to_owned();
        self.save(async move |s| s.delete_project(&id).await)
    }
    pub fn create_list(&mut self, project: &str, name: &str) -> Result<List> {
        let (project, name) = (project.to_owned(), name.to_owned());
        self.save(async move |s| s.create_list(&project, &name).await)
    }
    pub fn rename_list(&mut self, id: &str, name: &str) -> Result<()> {
        if let Some(l) = self.lists.iter_mut().find(|l| l.id == id) {
            l.name = name.to_owned();
        }
        let (id, name) = (id.to_owned(), name.to_owned());
        self.save(async move |s| s.rename_list(&id, &name).await)
    }
    pub fn reorder_list(&mut self, id: &str, sort_order: f64) -> Result<()> {
        let id = id.to_owned();
        self.save(async move |s| s.reorder_list(&id, sort_order).await)
    }
    pub fn delete_list(&mut self, id: &str) -> Result<()> {
        self.lists.retain(|l| l.id != id);
        self.tasks.retain(|t| t.list_id != id);
        let id = id.to_owned();
        self.save(async move |s| s.delete_list(&id).await)
    }
    pub fn create_task(&mut self, list: &str, title: &str, notes: Option<&str>, parent: Option<&str>) -> Result<Task> {
        let (list, title, notes, parent) = (list.to_owned(), title.to_owned(), notes.map(str::to_owned), parent.map(str::to_owned));
        self.save(async move |s| s.create_task(&list, &title, notes.as_deref(), parent.as_deref()).await)
    }
    pub fn update_task(&mut self, id: &str, title: Option<&str>, notes: Option<&str>, done: Option<bool>) -> Result<Task> {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            if let Some(v) = title {
                t.title = v.to_owned();
            }
            if let Some(v) = done {
                t.done = v;
            }
        }
        let (id, title, notes) = (id.to_owned(), title.map(str::to_owned), notes.map(str::to_owned));
        self.save(async move |s| s.update_task(&id, title.as_deref(), notes.as_deref(), done).await)
    }
    /// Re-homes a task under a parent (None: top level) in a list (None: its current one).
    pub fn move_task(&mut self, id: &str, parent: Option<&str>, list: Option<&str>, sort_order: Option<f64>) -> Result<Task> {
        let (id, parent, list) = (id.to_owned(), parent.map(str::to_owned), list.map(str::to_owned));
        self.save(async move |s| s.move_task(&id, parent.as_deref(), list.as_deref(), sort_order).await)
    }
    /// Moves tasks (and their subtrees) to the Trash; returns the ids that went, for Undo.
    pub fn trash_tasks(&mut self, ids: &[Id]) -> Result<Vec<Id>> {
        self.tasks.retain(|t| !ids.contains(&t.id));
        let ids = ids.to_vec();
        self.save(async move |s| s.soft_delete_tasks(&ids).await)
    }
    pub fn restore_tasks(&mut self, ids: &[Id]) -> Result<()> {
        self.trash.retain(|t| !ids.contains(&t.id));
        let ids = ids.to_vec();
        self.save(async move |s| s.restore_tasks(&ids).await)
    }
    pub fn purge_tasks(&mut self, ids: &[Id]) -> Result<()> {
        self.trash.retain(|t| !ids.contains(&t.id));
        let ids = ids.to_vec();
        self.save(async move |s| s.purge_tasks(&ids).await)
    }
    pub fn empty_trash(&mut self) -> Result<()> {
        self.trash.clear();
        self.save(async move |s| s.empty_trash().await)
    }
    pub fn create_note(&mut self, task: &str, content: &str) -> Result<Note> {
        let (task, content) = (task.to_owned(), content.to_owned());
        self.save(async move |s| s.create_note(&task, &content).await)
    }
    pub fn update_note(&mut self, id: &str, content: &str) -> Result<Note> {
        let (id, content) = (id.to_owned(), content.to_owned());
        self.save(async move |s| s.update_note(&id, &content).await)
    }
    pub fn delete_note(&mut self, id: &str) -> Result<()> {
        let id = id.to_owned();
        self.save(async move |s| s.delete_note(&id).await)
    }
    pub fn add_link(&mut self, task: &str, name: &str, url: &str) -> Result<Attachment> {
        let (task, name, url) = (task.to_owned(), name.to_owned(), url.to_owned());
        self.save(async move |s| s.add_attachment(&task, &name, LinkType::Reference, &url).await)
    }
    pub fn add_file(&mut self, task: &str, path: &str) -> Result<Attachment> {
        let (task, path) = (task.to_owned(), path.to_owned());
        self.save(async move |s| taskscape_storage::attachments::attach_copy(s, &task, &path, None).await)
    }
    pub fn rename_attachment(&mut self, id: &str, name: &str) -> Result<Attachment> {
        let (id, name) = (id.to_owned(), name.to_owned());
        self.save(async move |s| taskscape_storage::attachments::rename_attachment(s, &id, &name).await)
    }
    pub fn delete_attachment(&mut self, id: &str) -> Result<()> {
        let id = id.to_owned();
        self.save(async move |s| s.delete_attachment(&id).await)
    }
    pub fn set_setting(&mut self, key: &str, value: &str) -> Result<()> {
        let (key, value) = (key.to_owned(), value.to_owned());
        self.save(async move |s| s.set_setting(&key, &value).await)
    }
    pub fn get_setting(&self, key: &str) -> Option<String> {
        self.rt.block_on(self.store.get_setting(key)).ok().flatten()
    }
    /// The store itself, for what has no method here (hotkeys, attachment paths).
    pub fn with_store<T>(&self, f: impl AsyncFnOnce(&Store) -> Result<T>) -> Result<T> {
        self.rt.block_on(f(&self.store))
    }
}

static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
