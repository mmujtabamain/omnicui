//! Demo data: the projects, lists and tasks the reference screenshots show, the same as the web
//! version's (apps/web/backend/planner/demo), so one set of screen contracts holds on both.
//! `load(store, "standard" | "empty" | "nolists" | "long")` replaces everything in the store.
use anyhow::{bail, Result};
use sea_orm::{ActiveModelTrait, ConnectionTrait, Set};

use crate::entities::{attachments, lists, projects, task_notes, tasks};
use crate::models::LinkType;
use crate::storage::Store;
use crate::util::{new_id, now_millis};

const MIN: i64 = 60_000;
const DAY: i64 = 24 * 60 * MIN;

/// One task: key, list, title, parent key, notes, done, age, trashed this long ago.
struct T(&'static str, &'static str, &'static str, Option<&'static str>, &'static str, bool, i64, Option<i64>);

const STANDARD_TASKS: &[T] = &[
    T("pharmacy", "Errands", "Pick up prescription", None, "Counter closes at 6", false, DAY, None),
    T("post", "Errands", "Post the parcel to Mum", None, "", true, 3 * DAY, None),
    T("bike", "Errands", "Book bike service", None, "", false, 5 * DAY, None),
    T("keys", "Errands", "Cut a spare key", None, "", false, DAY, None),
    // Last in Errands, as the reference screens show it.
    T("groceries", "Errands", "Buy groceries for the week", None, "", false, 2 * DAY, None),
    T("milk", "Errands", "Oat milk", Some("groceries"), "", true, DAY, None),
    T("bread", "Errands", "Sourdough loaf", Some("groceries"), "", false, DAY, None),
    T("apples", "Errands", "Apples", Some("groceries"), "", false, DAY, None),
    T("novel", "Reading", "Finish the novel on the nightstand", None, "", false, DAY, None),
    T("article", "Reading", "Read the long article on soil health", None, "", true, DAY, None),
    T("library", "Reading", "Return library books", None, "", false, DAY, None),
    T("seeds", "Garden", "Order tomato seeds", None, "", false, DAY, None),
    T("hedge", "Garden", "Trim the hedge", None, "", false, DAY, None),
    T("hero", "Website", "Rewrite the hero section", None, "", false, 4 * DAY, None),
    T("copy", "Website", "Draft headline options", Some("hero"), "", false, DAY, None),
    T("shot", "Website", "Pick the hero image", Some("hero"), "", true, DAY, None),
    T("pricing", "Website", "Pricing page: compare plans table", None, "", false, DAY, None),
    T("faq", "Website", "Collect FAQ questions from support", None, "", false, DAY, None),
    T("logo", "Press kit", "Export logos in SVG and PNG", None, "", false, DAY, None),
    T("bios", "Press kit", "Team bios, 50 words each", None, "", true, DAY, None),
    T("old-flyer", "Errands", "Print the old flyer", None, "", false, DAY, Some(2 * DAY)),
    T("old-call", "Website", "Call the printer about samples", None, "", false, DAY, Some(30 * MIN)),
];

const NOTES: &[(&str, &str)] = &[
    ("hero", "<p>Lead with the <strong>one-line promise</strong>, then three proof points.</p><ul><li>Fast capture</li><li>Works offline</li><li>Keyboard first</li></ul>"),
    ("hero", "<p>Ask design for two layouts by Friday.</p>"),
    ("groceries", "<p>Check the cupboard before going.</p>"),
];

const HERO_SKETCH: &[u8] = include_bytes!("../demo/hero-sketch.png");

/// Replaces everything in the store with the named data set.
pub async fn load(store: &Store, name: &str) -> Result<()> {
    let db = store.db();
    for table in ["attachments", "task_notes", "tasks", "lists", "projects"] {
        db.execute_unprepared(&format!("DELETE FROM {table}")).await?;
    }
    let now = now_millis();
    let (projects_, lists_, standard): (&[(&str, i64)], &[(&str, &str)], bool) = match name {
        "standard" | "long" => (
            &[("Home", 30 * DAY), ("Spring launch", 20 * DAY)],
            &[("Home", "Errands"), ("Home", "Reading"), ("Home", "Garden"), ("Spring launch", "Website"), ("Spring launch", "Press kit")],
            true,
        ),
        "empty" => (&[("Home", DAY)], &[("Home", "Inbox")], false),
        "nolists" => (&[("Home", DAY)], &[], false),
        other => bail!("no demo data set {other}"),
    };

    let mut project_ids = std::collections::HashMap::new();
    for (pname, age) in projects_ {
        let id = new_id();
        projects::ActiveModel { id: Set(id.clone()), name: Set(pname.to_string()), created_at: Set(now - age), updated_at: Set(now - age) }
            .insert(db)
            .await?;
        project_ids.insert(*pname, id);
    }
    let mut list_ids = std::collections::HashMap::new();
    let mut per_project = std::collections::HashMap::new();
    for (pname, lname) in lists_ {
        let n: &mut i32 = per_project.entry(*pname).or_insert(0);
        lists::ActiveModel {
            id: Set(new_id()),
            project_id: Set(project_ids[pname].clone()),
            name: Set(lname.to_string()),
            sort_order: Set(*n as f64),
            created_at: Set(now - DAY),
            updated_at: Set(now - DAY),
        }
        .insert(db)
        .await
        .map(|m| list_ids.insert(*lname, m.id))?;
        *n += 1;
    }

    let mut all: Vec<T> = vec![];
    if standard {
        all.extend(STANDARD_TASKS.iter().map(|t| T(t.0, t.1, t.2, t.3, t.4, t.5, t.6, t.7)));
    }
    let long_titles: Vec<String> = (1..=40).map(|i| format!("Errand number {i}")).collect();
    let mut task_ids = std::collections::HashMap::new();
    let mut order = 0.0;
    let mut insert = |key: String, list: &str, title: String, parent: Option<&str>, notes: &str, done: bool, age: i64, trashed: Option<i64>| {
        let id = new_id();
        let model = tasks::ActiveModel {
            id: Set(id.clone()),
            list_id: Set(list_ids[list].clone()),
            parent_id: Set(parent.map(|p| task_ids.get(p).cloned().expect("a parent before its subtasks"))),
            title: Set(title),
            notes: Set(if notes.is_empty() { None } else { Some(notes.to_string()) }),
            done: Set(done as i32),
            sort_order: Set(order),
            created_at: Set(now - age),
            updated_at: Set(now - age),
            deleted_at: Set(trashed.map(|t| now - t)),
        };
        order += 1.0;
        task_ids.insert(key, id);
        model
    };
    let mut models = vec![];
    for t in &all {
        models.push(insert(t.0.to_string(), t.1, t.2.to_string(), t.3, t.4, t.5, t.6, t.7));
    }
    if name == "long" {
        for (i, title) in long_titles.into_iter().enumerate() {
            let i = i + 1;
            models.push(insert(format!("long-{i}"), "Errands", title, None, "", i % 5 == 1, DAY, None));
        }
    }
    for m in models {
        m.insert(db).await?;
    }

    if standard {
        for (i, (key, content)) in NOTES.iter().enumerate() {
            task_notes::ActiveModel {
                id: Set(new_id()),
                task_id: Set(task_ids[*key].clone()),
                content: Set(content.to_string()),
                sort_order: Set(i as f64),
                created_at: Set(now - DAY),
                updated_at: Set(now - DAY),
            }
            .insert(db)
            .await?;
        }
        let hero = task_ids["hero"].clone();
        attachments::ActiveModel {
            id: Set(new_id()),
            task_id: Set(hero.clone()),
            name: Set("Brand brief".into()),
            link_type: Set(LinkType::Reference.as_str().into()),
            location: Set("https://example.com/brand-brief".into()),
            created_at: Set(now - DAY),
        }
        .insert(db)
        .await?;
        crate::paths::ensure_dirs()?;
        let stored = format!("{}-hero-sketch.png", new_id());
        std::fs::write(crate::paths::attachments_dir().join(&stored), HERO_SKETCH)?;
        attachments::ActiveModel {
            id: Set(new_id()),
            task_id: Set(hero),
            name: Set("hero-sketch.png".into()),
            link_type: Set(LinkType::Copy.as_str().into()),
            location: Set(format!("attachments/{stored}")),
            created_at: Set(now - DAY + 1),
        }
        .insert(db)
        .await?;
    }

    // The window opens on the last list, as the reference remembers it across launches.
    let last = if standard { ("Home", Some("Errands")) } else if name == "empty" { ("Home", Some("Inbox")) } else { ("Home", None) };
    store.set_setting("last_active_project", &project_ids[last.0]).await?;
    match last.1 {
        Some(l) => store.set_setting("last_active_list", &list_ids[l]).await?,
        None => store.set_setting("last_active_list", "").await?,
    }
    Ok(())
}
