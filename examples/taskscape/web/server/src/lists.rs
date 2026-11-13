//! A list as JSON (the desktop app's export format, so a file moves between the two), and a
//! selection of tasks as a Markdown checklist. Ported from `tauri/main/src-tauri/src/lib.rs`
//! (`export_list`, `import_list`, `copy_tasks`), which keeps them in the Tauri app, not the
//! common crate; the web reads and writes the document in the browser instead of at a path.

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use taskscape_common::{attachments, storage::strip_html, LinkType, List, Store, Task};

#[derive(Serialize, Deserialize)]
struct ListExport {
    taskscape_export: u32,
    list: ListMeta,
    tasks: Vec<ExportTask>,
}

#[derive(Serialize, Deserialize)]
struct ListMeta {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct ExportTask {
    title: String,
    done: bool,
    #[serde(default)]
    notes: Vec<String>,
    #[serde(default)]
    attachments: Vec<ExportAttachment>,
    #[serde(default)]
    children: Vec<ExportTask>,
}

#[derive(Serialize, Deserialize)]
struct ExportAttachment {
    name: String,
    location: String,
}

type Children<'a> = HashMap<&'a str, Vec<&'a Task>>;

fn group_by_parent(tasks: &[Task]) -> (Vec<&Task>, Children<'_>) {
    let mut roots = Vec::new();
    let mut children: Children = HashMap::new();
    for t in tasks {
        match &t.parent_id {
            Some(p) => children.entry(p.as_str()).or_default().push(t),
            None => roots.push(t),
        }
    }
    (roots, children)
}

fn to_export(node: &Task, children: &Children) -> ExportTask {
    ExportTask {
        title: node.title.clone(),
        done: node.done,
        notes: node.note_items.iter().map(|n| n.content.clone()).collect(),
        attachments: node
            .attachments
            .iter()
            .filter(|a| a.link_type == LinkType::Reference)
            .map(|a| ExportAttachment { name: a.name.clone(), location: a.location.clone() })
            .collect(),
        children: children
            .get(node.id.as_str())
            .map(|kids| kids.iter().map(|k| to_export(k, children)).collect())
            .unwrap_or_default(),
    }
}

/// The list's name and its export document.
pub async fn export(store: &Store, list_id: &str) -> Result<(String, Value)> {
    let list = store
        .list_lists()
        .await?
        .into_iter()
        .find(|l| l.id == list_id)
        .ok_or_else(|| anyhow!("list not found: {list_id}"))?;
    let tasks = store.list_tasks(list_id).await?;
    let (roots, children) = group_by_parent(&tasks);
    let doc = ListExport {
        taskscape_export: 1,
        list: ListMeta { name: list.name.clone() },
        tasks: roots.iter().map(|r| to_export(r, &children)).collect(),
    };
    Ok((list.name, serde_json::to_value(doc)?))
}

async fn import_task(store: &Store, list_id: &str, parent: Option<&str>, t: &ExportTask) -> Result<()> {
    let task = store.create_task(list_id, &t.title, None, parent).await?;
    if t.done {
        store.update_task(&task.id, None, None, Some(true)).await?;
    }
    for note in &t.notes {
        store.create_note(&task.id, note).await?;
    }
    for a in &t.attachments {
        attachments::attach_reference(store, &task.id, &a.name, &a.location).await?;
    }
    for child in &t.children {
        Box::pin(import_task(store, list_id, Some(&task.id), child)).await?;
    }
    Ok(())
}

/// A new list in `project_id` from an export document; `name`, when not blank, replaces the
/// document's own.
pub async fn import(store: &Store, project_id: &str, doc: Value, name: Option<String>) -> Result<List> {
    let export: ListExport = serde_json::from_value(doc).map_err(|e| anyhow!("not a Taskscape list export: {e}"))?;
    let name = name.filter(|s| !s.trim().is_empty()).unwrap_or(export.list.name);
    let list = store.create_list(project_id, &name).await?;
    for t in &export.tasks {
        import_task(store, &list.id, None, t).await?;
    }
    Ok(list)
}

/// The selected tasks as a Markdown checklist in tree order: a task indented two spaces for each
/// of its ancestors that is also selected, each of its notes as a quoted line under it.
pub fn copy_markdown(tasks: &[Task], ids: &[String]) -> String {
    let selected: HashSet<&str> = ids.iter().map(String::as_str).collect();
    let (roots, children) = group_by_parent(tasks);
    fn render(node: &Task, depth: usize, children: &Children, selected: &HashSet<&str>, out: &mut String) {
        let picked = selected.contains(node.id.as_str());
        if picked {
            out.push_str(&"  ".repeat(depth));
            out.push_str(if node.done { "- [x] " } else { "- [ ] " });
            out.push_str(&node.title);
            out.push('\n');
            for note in &node.note_items {
                let text = strip_html(&note.content);
                if !text.is_empty() {
                    out.push_str(&"  ".repeat(depth + 1));
                    out.push_str("> ");
                    out.push_str(&text);
                    out.push('\n');
                }
            }
        }
        let depth = depth + usize::from(picked);
        for k in children.get(node.id.as_str()).into_iter().flatten() {
            render(k, depth, children, selected, out);
        }
    }
    let mut out = String::new();
    for r in &roots {
        render(r, 0, &children, &selected, &mut out);
    }
    out.trim_end().to_string()
}
