//! The Taskscape example's fixtures, loaded through the app's own `Store`.
//!
//! Every version of Taskscape that omnicui checks reads the same data the same way: a fixture
//! (`../<seed>.json`) is written into a fresh, empty data folder with `taskscape_common`'s
//! `Store` and nothing else, so ids, timestamps, ordering keys and derived fields (a task's
//! plaintext `notes`) are exactly what the app itself would have written.
//!
//! `Store::open` finds its data folder from the home directory (`~/.taskscape/`), and the
//! settings file is read from there on every call, so `load` points `HOME` at a new temporary
//! folder before it opens the store. That makes `HOME` process-wide state: load one fixture at a
//! time, and don't read the real home folder from the same process afterwards.
//!
//! The store's ids are random (UUID v4). `load` returns the fixture's keys mapped to them
//! (`keys`), so a registry can report events in the fixture's own names (`"brief"`, `"launch"`)
//! and a test can ask for them by name. Ids the fixture didn't name (a task created during a
//! test) stay as the store made them.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use taskscape_common::{attachments, Store};

/// The fixtures, by seed name. Embedded, so a loader binary carries them wherever it runs.
pub const SEEDS: &[(&str, &str)] = &[
    ("standard", include_str!("../standard.json")),
    ("empty", include_str!("../empty.json")),
    ("nolists", include_str!("../nolists.json")),
    ("long", include_str!("../long.json")),
];

/// Files that fixture attachments copy in (`"copy": "<file>"`), by name.
const FILES: &[(&str, &[u8])] = &[("whiteboard.png", include_bytes!("../files/whiteboard.png"))];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    #[allow(dead_code)]
    about: String,
    projects: Vec<ProjectFx>,
    #[serde(default, rename = "extraTrashed")]
    extra_trashed: Vec<TrashedFx>,
    #[serde(default)]
    active: ActiveFx,
    /// Settings written before anything else (`show_completed`, `theme`, …).
    #[serde(default)]
    settings: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectFx {
    key: String,
    name: String,
    lists: Vec<ListFx>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ListFx {
    key: String,
    name: String,
    tasks: Vec<TaskFx>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskFx {
    key: String,
    title: String,
    #[serde(default)]
    done: bool,
    /// Rich-text note blocks, as the app stores them (sanitized HTML).
    #[serde(default)]
    notes: Vec<String>,
    #[serde(default)]
    attachments: Vec<AttachmentFx>,
    #[serde(default)]
    children: Vec<TaskFx>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AttachmentFx {
    name: String,
    /// A reference: the URL or path, stored as given.
    #[serde(default)]
    location: Option<String>,
    /// A copy: one of `FILES`, copied into the data folder as the app copies a file.
    #[serde(default)]
    copy: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrashedFx {
    list: String,
    key: String,
    title: String,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ActiveFx {
    project: Option<String>,
    list: Option<String>,
}

/// A loaded fixture: the open store, the fixture's keys mapped to the store's ids, and the
/// temporary home folder that holds `.taskscape/`.
pub struct Loaded {
    pub store: Store,
    pub keys: BTreeMap<String, String>,
    pub home: PathBuf,
}

/// The seed names, for messages.
pub fn seed_names() -> Vec<&'static str> {
    SEEDS.iter().map(|(name, _)| *name).collect()
}

/// Writes `seed` into a fresh data folder under `root` (created if missing) and returns the
/// open store. Sets `HOME` to that folder: see the module comment.
pub async fn load(seed: &str, root: &Path) -> Result<Loaded> {
    let json = SEEDS
        .iter()
        .find(|(name, _)| *name == seed)
        .map(|(_, json)| *json)
        .ok_or_else(|| anyhow!("no fixture `{seed}`; the fixtures are {}", seed_names().join(", ")))?;
    let fixture: Fixture =
        serde_json::from_str(json).with_context(|| format!("fixtures/{seed}.json"))?;

    std::fs::create_dir_all(root)?;
    let home = fresh_dir(root, seed)?;
    std::env::set_var("HOME", &home);
    let store = Store::open().await.context("opening the store")?;

    let mut loader = Loader { store: &store, keys: BTreeMap::new(), files: home.join("fixture-files") };
    for (key, value) in &fixture.settings {
        store.set_setting(key, value).await?;
    }
    for project in &fixture.projects {
        loader.project(project).await?;
    }
    for t in &fixture.extra_trashed {
        let list = loader.id(&t.list)?;
        let task = store.create_task(&list, &t.title, None, None).await?;
        loader.name(&t.key, &task.id)?;
        pause().await;
    }
    // Trash last, so the trashed tasks' timestamps are the newest ("deleted just now"), and one
    // at a time: the Trash lists the most recently deleted first, by the millisecond.
    for t in &fixture.extra_trashed {
        store.soft_delete_tasks(&[loader.id(&t.key)?]).await?;
        pause().await;
    }
    if let Some(p) = &fixture.active.project {
        store.set_setting("last_active_project", &loader.id(p)?).await?;
    }
    if let Some(l) = &fixture.active.list {
        store.set_setting("last_active_list", &loader.id(l)?).await?;
    }
    let keys = loader.keys;
    Ok(Loaded { store, keys, home })
}

struct Loader<'a> {
    store: &'a Store,
    keys: BTreeMap<String, String>,
    files: PathBuf,
}

impl Loader<'_> {
    fn name(&mut self, key: &str, id: &str) -> Result<()> {
        if self.keys.insert(key.to_string(), id.to_string()).is_some() {
            bail!("the fixture uses the key `{key}` twice");
        }
        Ok(())
    }

    fn id(&self, key: &str) -> Result<String> {
        self.keys.get(key).cloned().ok_or_else(|| anyhow!("the fixture names `{key}`, which it doesn't define"))
    }

    async fn project(&mut self, p: &ProjectFx) -> Result<()> {
        let project = self.store.create_project(&p.name).await?;
        self.name(&p.key, &project.id)?;
        // Projects are ordered by creation time alone, to the millisecond.
        pause().await;
        for (i, l) in p.lists.iter().enumerate() {
            let list = self.store.create_list(&project.id, &l.name).await?;
            self.store.reorder_list(&list.id, i as f64 + 1.0).await?;
            self.name(&l.key, &list.id)?;
            for (j, t) in l.tasks.iter().enumerate() {
                Box::pin(self.task(&list.id, None, j, t)).await?;
            }
        }
        Ok(())
    }

    async fn task(&mut self, list: &str, parent: Option<&str>, at: usize, t: &TaskFx) -> Result<()> {
        let task = self.store.create_task(list, &t.title, None, parent).await?;
        // The store orders by `sort_order` (seeded with the creation millisecond); fixed keys
        // make the fixture's order the order shown, however fast it was written.
        self.store.reorder_task(&task.id, at as f64 + 1.0).await?;
        self.name(&t.key, &task.id)?;
        if t.done {
            self.store.update_task(&task.id, None, None, Some(true)).await?;
        }
        for note in &t.notes {
            self.store.create_note(&task.id, note).await?;
            // Notes are ordered by their creation millisecond.
            pause().await;
        }
        for a in &t.attachments {
            match (&a.location, &a.copy) {
                (Some(location), None) => {
                    attachments::attach_reference(self.store, &task.id, &a.name, location).await?;
                }
                (None, Some(file)) => {
                    let path = self.file(file)?;
                    attachments::attach_copy(self.store, &task.id, &path.to_string_lossy(), Some(&a.name))
                        .await?;
                }
                _ => bail!("attachment `{}` needs exactly one of `location` and `copy`", a.name),
            }
            pause().await;
        }
        for (k, child) in t.children.iter().enumerate() {
            Box::pin(self.task(list, Some(&task.id), k, child)).await?;
        }
        Ok(())
    }

    /// Writes one of `FILES` out, so that `attach_copy` copies it in as it would a user's file.
    fn file(&self, name: &str) -> Result<PathBuf> {
        let bytes = FILES
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, b)| *b)
            .ok_or_else(|| anyhow!("no fixture file `{name}`"))?;
        std::fs::create_dir_all(&self.files)?;
        let path = self.files.join(name);
        std::fs::write(&path, bytes)?;
        Ok(path)
    }
}

/// Two milliseconds, so rows ordered by their creation time come out in the fixture's order.
async fn pause() {
    tokio::time::sleep(Duration::from_millis(2)).await;
}

/// A new, empty folder under `root`, named for the seed and unique within this process.
fn fresh_dir(root: &Path, seed: &str) -> Result<PathBuf> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    loop {
        let n = N.fetch_add(1, Ordering::SeqCst);
        let dir = root.join(format!("{seed}-{}-{n}", std::process::id()));
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e).with_context(|| format!("creating {}", dir.display())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // One test for every seed: `load` sets the process-wide `HOME`, so they can't run in parallel.
    #[tokio::test]
    async fn every_seed_loads_in_its_order_through_the_store() {
        let root = std::env::temp_dir().join(format!("taskscape-fixtures-test-{}", std::process::id()));

        let l = load("standard", &root).await.unwrap();
        let lists = l.store.list_lists().await.unwrap();
        let atlas: Vec<_> =
            lists.iter().filter(|x| x.project_id == l.keys["atlas"]).map(|x| x.name.as_str()).collect();
        assert_eq!(atlas, ["Launch plan", "Groceries", "Inbox"]);
        let projects = l.store.list_projects().await.unwrap();
        assert_eq!(projects.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["Atlas", "Home"]);
        let launch = l.store.list_tasks(&l.keys["launch"]).await.unwrap();
        let roots: Vec<_> = launch.iter().filter(|t| t.parent_id.is_none()).map(|t| t.title.as_str()).collect();
        assert_eq!(roots, ["Write the launch brief", "Book the venue", "Order the banners", "Send the invitations"]);
        assert_eq!(launch.len(), 8, "four tasks and four subtasks");
        let brief = l.store.get_task(&l.keys["brief"]).await.unwrap();
        assert_eq!(brief.notes.as_deref(), Some("Draft due Friday ; keep it to one page."));
        assert_eq!(brief.attachments.len(), 2);
        assert!(l.home.join(".taskscape").join(&brief.attachments[1].location).exists(), "the copy is in the data folder");
        assert!(l.store.get_task(&l.keys["venue"]).await.unwrap().done);
        let trashed = l.store.list_trashed().await.unwrap();
        let titles: Vec<_> = trashed.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, ["Paint swatches", "Old launch draft"], "newest first");
        assert_eq!(l.store.get_setting("last_active_list").await.unwrap(), Some(l.keys["launch"].clone()));

        let l = load("empty", &root).await.unwrap();
        assert_eq!(l.store.all_tasks().await.unwrap().len(), 0, "a fresh folder: nothing from the last seed");
        assert_eq!(l.store.list_lists().await.unwrap().len(), 1);

        let l = load("nolists", &root).await.unwrap();
        assert!(l.store.list_lists().await.unwrap().is_empty());

        let l = load("long", &root).await.unwrap();
        let reading = l.store.list_tasks(&l.keys["reading"]).await.unwrap();
        assert_eq!(reading.len(), 30);
        assert_eq!(reading[0].title, "Read chapter 1");
        assert_eq!(reading[29].title, "Read chapter 30");

        assert!(load("nonesuch", &root).await.is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}
