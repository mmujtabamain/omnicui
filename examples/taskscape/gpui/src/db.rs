//! The store: `taskscape_common`'s `Store`, linked directly (no HTTP, no IPC). Its API is async
//! (SeaORM over SQLx); SQLite answers in well under a frame, so the UI waits for it in place.

use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use taskscape_common::{hotkeys, hotkeys::ResolvedBinding, List, Project, Store, Task};

pub struct Db {
    pub store: Store,
    rt: Arc<tokio::runtime::Runtime>,
}

/// Everything the window shows, read in one go.
#[derive(Clone, Default)]
pub struct Snapshot {
    pub projects: Vec<Project>,
    pub lists: Vec<List>,
    pub tasks: Vec<Task>,
    pub trashed: Vec<Task>,
}

impl Db {
    pub fn new(store: Store, rt: Arc<tokio::runtime::Runtime>) -> Db {
        Db { store, rt }
    }

    /// Opens the user's own store (`~/.taskscape`).
    pub fn open() -> Result<Db> {
        let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()?);
        let store = rt.block_on(Store::open())?;
        Ok(Db { store, rt })
    }

    pub fn run<T>(&self, f: impl Future<Output = Result<T>>) -> Result<T> {
        let r = self.rt.block_on(f);
        if let Err(e) = &r {
            eprintln!("[taskscape] store: {e:#}");
        }
        r
    }

    pub fn snapshot(&self) -> Snapshot {
        let s = &self.store;
        Snapshot {
            projects: self.run(s.list_projects()).unwrap_or_default(),
            lists: self.run(s.list_lists()).unwrap_or_default(),
            tasks: self.run(s.all_tasks()).unwrap_or_default(),
            trashed: self.run(s.list_trashed()).unwrap_or_default(),
        }
    }

    pub fn setting(&self, key: &str) -> Option<String> {
        self.run(self.store.get_setting(key)).ok().flatten()
    }

    pub fn set_setting(&self, key: &str, value: &str) {
        let _ = self.run(self.store.set_setting(key, value));
    }

    pub fn hotkeys(&self) -> Vec<ResolvedBinding> {
        self.run(hotkeys::resolve(&self.store)).unwrap_or_default()
    }
}
