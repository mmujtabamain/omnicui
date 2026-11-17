//! A list as a JSON document, the reference's export format: its tasks with their subtasks, notes and
//! reference attachments (copied files live in the data folder and do not travel). Import makes a new list.
use crate::models::{LinkType, List, Task};
use crate::storage::Store;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct ListExport {
    /// Schema version, so a future import can migrate older documents.
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

fn to_export(node: &Task, children: &HashMap<&str, Vec<&Task>>) -> ExportTask {
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
        children: children.get(node.id.as_str()).map(|kids| kids.iter().map(|k| to_export(k, children)).collect()).unwrap_or_default(),
    }
}

/// The list as the reference's JSON document.
pub async fn export_list(store: &Store, list_id: &str) -> Result<String> {
    let list = store.list_lists().await?.into_iter().find(|l| l.id == list_id).ok_or_else(|| anyhow!("list not found: {list_id}"))?;
    let tasks = store.list_tasks(list_id).await?;
    let mut children: HashMap<&str, Vec<&Task>> = HashMap::new();
    let mut roots: Vec<&Task> = vec![];
    for t in &tasks {
        match t.parent_id.as_deref() {
            Some(p) => children.entry(p).or_default().push(t),
            None => roots.push(t),
        }
    }
    for v in children.values_mut() {
        v.sort_by(|a, b| a.sort_order.total_cmp(&b.sort_order));
    }
    roots.sort_by(|a, b| a.sort_order.total_cmp(&b.sort_order));
    let doc = ListExport { taskscape_export: 1, list: ListMeta { name: list.name }, tasks: roots.iter().map(|r| to_export(r, &children)).collect() };
    Ok(serde_json::to_string_pretty(&doc)?)
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
        crate::attachments::attach_reference(store, &task.id, &a.name, &a.location).await?;
    }
    for child in &t.children {
        Box::pin(import_task(store, list_id, Some(&task.id), child)).await?;
    }
    Ok(())
}

/// A new list in the project from an exported document; `name` overrides the document's when not blank.
pub async fn import_list(store: &Store, project_id: &str, json: &str, name: Option<&str>) -> Result<List> {
    let doc: ListExport = serde_json::from_str(json)?;
    let list_name = name.filter(|s| !s.trim().is_empty()).map(str::to_owned).unwrap_or(doc.list.name);
    let list = store.create_list(project_id, &list_name).await?;
    for t in &doc.tasks {
        import_task(store, &list.id, None, t).await?;
    }
    Ok(list)
}
