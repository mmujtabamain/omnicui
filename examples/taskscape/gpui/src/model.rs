//! The product's rules over the store's data: the task tree, completion, copying, search and
//! the filter view (SPEC-PRODUCT.md, "taskscape.pane" and "taskscape.filter"). No drawing here.

use std::collections::{HashMap, HashSet};

use taskscape_common::{storage::strip_html, Task};

/// The tasks of one list, as a tree.
pub struct Tree<'a> {
    pub roots: Vec<&'a Task>,
    pub children: HashMap<&'a str, Vec<&'a Task>>,
    pub by_id: HashMap<&'a str, &'a Task>,
}

impl<'a> Tree<'a> {
    /// `tasks` in the store's order (sort_order, then created).
    pub fn new(tasks: impl IntoIterator<Item = &'a Task>) -> Tree<'a> {
        let all: Vec<&Task> = tasks.into_iter().collect();
        let by_id: HashMap<&str, &Task> = all.iter().map(|t| (t.id.as_str(), *t)).collect();
        let mut roots = Vec::new();
        let mut children: HashMap<&str, Vec<&Task>> = HashMap::new();
        for t in &all {
            match t.parent_id.as_deref() {
                Some(p) if by_id.contains_key(p) => children.entry(p).or_default().push(t),
                _ => roots.push(*t),
            }
        }
        Tree { roots, children, by_id }
    }

    pub fn kids(&self, id: &str) -> &[&'a Task] {
        self.children.get(id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Every task under `id`, depth first, in order.
    pub fn descendants(&self, id: &str) -> Vec<&'a Task> {
        let mut out = Vec::new();
        for k in self.kids(id) {
            out.push(*k);
            out.extend(self.descendants(&k.id));
        }
        out
    }

    pub fn ancestors(&self, id: &str) -> Vec<&'a Task> {
        let mut out = Vec::new();
        let mut cur = self.by_id.get(id).and_then(|t| t.parent_id.as_deref());
        while let Some(p) = cur {
            match self.by_id.get(p) {
                Some(t) => {
                    out.push(*t);
                    cur = t.parent_id.as_deref();
                }
                None => break,
            }
        }
        out
    }

    pub fn depth(&self, id: &str) -> usize {
        self.ancestors(id).len()
    }

    /// The whole tree, depth first.
    pub fn flat(&self) -> Vec<&'a Task> {
        let mut out = Vec::new();
        for r in &self.roots {
            out.push(*r);
            out.extend(self.descendants(&r.id));
        }
        out
    }

    pub fn siblings(&self, t: &Task) -> Vec<&'a Task> {
        match t.parent_id.as_deref().filter(|p| self.by_id.contains_key(p)) {
            Some(p) => self.kids(p).to_vec(),
            None => self.roots.clone(),
        }
    }

    /// What changes when `id` is marked `done`, in order: the task, then its subtree (only the
    /// tasks that change); completing also completes each ancestor whose children are then all
    /// done; reopening reopens every ancestor that was done.
    pub fn completion(&self, id: &str, done: bool) -> Vec<(String, bool)> {
        let Some(t) = self.by_id.get(id) else { return vec![] };
        let mut state: HashMap<&str, bool> = self.by_id.iter().map(|(k, t)| (*k, t.done)).collect();
        let mut out = Vec::new();
        let mut set = |tid: &'a str, out: &mut Vec<(String, bool)>, state: &mut HashMap<&'a str, bool>| {
            if state.get(tid) != Some(&done) {
                state.insert(tid, done);
                out.push((tid.to_string(), done));
            }
        };
        set(&t.id, &mut out, &mut state);
        for d in self.descendants(id) {
            set(&d.id, &mut out, &mut state);
        }
        for a in self.ancestors(id) {
            if done {
                let all_done = self.kids(&a.id).iter().all(|k| state.get(k.id.as_str()) == Some(&true));
                if !all_done {
                    break;
                }
            }
            set(&a.id, &mut out, &mut state);
        }
        out
    }
}

/// `copy_tasks` (the app's `main/src-tauri/src/lib.rs`): the selected tasks as a Markdown
/// checklist in tree order, indented by how many selected ancestors each has, notes quoted.
pub fn copy_markdown(tasks: &[Task], ids: &[String]) -> String {
    let selected: HashSet<&str> = ids.iter().map(String::as_str).collect();
    let tree = Tree::new(tasks.iter());
    fn render(tree: &Tree, node: &Task, depth: usize, selected: &HashSet<&str>, out: &mut String) {
        let picked = selected.contains(node.id.as_str());
        if picked {
            out.push_str(&"  ".repeat(depth));
            out.push_str(if node.done { "- [x] " } else { "- [ ] " });
            out.push_str(&node.title);
            out.push('\n');
            for note in &node.note_items {
                let text = strip_html(&note.content);
                if text.is_empty() {
                    continue;
                }
                out.push_str(&"  ".repeat(depth + 1));
                out.push_str("> ");
                out.push_str(&text);
                out.push('\n');
            }
        }
        for k in tree.kids(&node.id) {
            render(tree, k, depth + usize::from(picked), selected, out);
        }
    }
    let mut out = String::new();
    for r in &tree.roots {
        render(&tree, r, 0, &selected, &mut out);
    }
    out.trim_end().to_string()
}

/// "just now", "12m ago", "3h ago", "yesterday", "3d ago", "2w ago", then a date.
pub fn relative(ms: i64, now: i64) -> String {
    let s = ((now - ms) / 1000).max(0);
    if s < 45 {
        return "just now".into();
    }
    let m = (s + 30) / 60;
    if m < 60 {
        return format!("{}m ago", m.max(1));
    }
    let h = s / 3600;
    if h < 24 {
        return format!("{h}h ago");
    }
    let d = s / 86400;
    if d < 2 {
        return "yesterday".into();
    }
    if d < 7 {
        return format!("{d}d ago");
    }
    if d < 28 {
        return format!("{}w ago", d / 7);
    }
    let days = ms / 86_400_000;
    let (y, mo, da) = civil(days);
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    format!("{} {da}, {y}", MONTHS[(mo - 1) as usize])
}

fn civil(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Where search looks.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SearchFields {
    #[default]
    Both,
    Title,
    Notes,
}

pub fn matches(t: &Task, q: &str, fields: SearchFields) -> bool {
    let q = q.trim().to_lowercase();
    if q.is_empty() {
        return false;
    }
    let title = || t.title.to_lowercase().contains(&q);
    let notes = || t.notes.as_deref().unwrap_or("").to_lowercase().contains(&q);
    match fields {
        SearchFields::Both => title() || notes(),
        SearchFields::Title => title(),
        SearchFields::Notes => notes(),
    }
}

/// The filter panel's view of a list (`taskscape.filter`): display only.
#[derive(Clone, PartialEq, Debug)]
pub struct View {
    pub sort: Sort,
    pub asc: bool,
    pub show: Show,
    pub notes: Has,
    pub attachments: Has,
    pub subtasks: Has,
    pub date_field_updated: bool,
    pub within_days: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sort {
    Manual,
    Created,
    Updated,
    Alpha,
    DoneLast,
}
impl Sort {
    pub fn id(self) -> &'static str {
        match self {
            Sort::Manual => "manual",
            Sort::Created => "created",
            Sort::Updated => "updated",
            Sort::Alpha => "alpha",
            Sort::DoneLast => "doneLast",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Sort::Manual => "Manual",
            Sort::Created => "Date created",
            Sort::Updated => "Date updated",
            Sort::Alpha => "Alphabetical",
            Sort::DoneLast => "Done last",
        }
    }
    pub const ALL: [Sort; 5] = [Sort::Manual, Sort::Created, Sort::Updated, Sort::Alpha, Sort::DoneLast];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Show {
    All,
    Active,
    Completed,
}
impl Show {
    pub fn id(self) -> &'static str {
        match self {
            Show::All => "all",
            Show::Active => "active",
            Show::Completed => "completed",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Has {
    Any,
    With,
    Without,
}

impl Default for View {
    fn default() -> Self {
        View { sort: Sort::Manual, asc: true, show: Show::All, notes: Has::Any, attachments: Has::Any, subtasks: Has::Any, date_field_updated: false, within_days: None }
    }
}

impl View {
    pub fn is_default(&self) -> bool {
        *self == View::default()
    }

    /// Whether a task passes the view's filters on its own.
    pub fn keeps(&self, t: &Task, tree: &Tree, now: i64) -> bool {
        let show = match self.show {
            Show::All => true,
            Show::Active => !t.done,
            Show::Completed => t.done,
        };
        let has = |h: Has, yes: bool| match h {
            Has::Any => true,
            Has::With => yes,
            Has::Without => !yes,
        };
        let notes = !t.note_items.is_empty() || t.notes.as_deref().is_some_and(|n| !n.trim().is_empty());
        let date = match self.within_days {
            None => true,
            Some(d) => {
                let at = if self.date_field_updated { t.updated_at } else { t.created_at };
                now - at <= d as i64 * 86_400_000
            }
        };
        show && has(self.notes, notes) && has(self.attachments, !t.attachments.is_empty()) && has(self.subtasks, !tree.kids(&t.id).is_empty()) && date
    }

    pub fn order<'a>(&self, mut v: Vec<&'a Task>) -> Vec<&'a Task> {
        match self.sort {
            Sort::Manual => {}
            Sort::Created => v.sort_by_key(|t| t.created_at),
            Sort::Updated => v.sort_by_key(|t| t.updated_at),
            Sort::Alpha => v.sort_by_key(|t| t.title.to_lowercase()),
            Sort::DoneLast => v.sort_by_key(|t| t.done),
        }
        if !self.asc && self.sort != Sort::Manual {
            v.reverse();
        }
        v
    }
}

/// A row the pane shows.
#[derive(Clone, Debug)]
pub struct Row {
    pub id: String,
    pub depth: usize,
    pub has_kids: bool,
    pub expanded: bool,
    pub matched: bool,
}

/// The rows of a list: each top-level task followed by its subtasks, unless collapsed; done tasks
/// hidden when Show completed is off; while searching, the matches and their ancestors; under a
/// view, the tasks it keeps and their ancestors.
pub fn rows(tree: &Tree, collapsed: &HashSet<String>, show_completed: bool, search: Option<(&str, SearchFields)>, view: &View, now: i64) -> Vec<Row> {
    let searching = search.filter(|(q, _)| !q.trim().is_empty());
    let filtering = !view.is_default();
    // Which tasks are kept by search or the view: they and their ancestors are shown.
    let mut keep: Option<HashSet<&str>> = None;
    let mut hits: HashSet<&str> = HashSet::new();
    if searching.is_some() || filtering {
        let mut k = HashSet::new();
        for t in tree.flat() {
            let s = searching.map_or(true, |(q, f)| matches(t, q, f));
            let v = !filtering || view.keeps(t, tree, now);
            if s && v {
                if searching.is_some() {
                    hits.insert(t.id.as_str());
                }
                k.insert(t.id.as_str());
                for a in tree.ancestors(&t.id) {
                    k.insert(a.id.as_str());
                }
            }
        }
        keep = Some(k);
    }
    let mut out = Vec::new();
    fn walk<'a>(
        tree: &Tree<'a>,
        list: &[&'a Task],
        depth: usize,
        collapsed: &HashSet<String>,
        show_completed: bool,
        keep: &Option<HashSet<&str>>,
        hits: &HashSet<&str>,
        forced_open: bool,
        view: &View,
        out: &mut Vec<Row>,
    ) {
        for t in view.order(list.to_vec()) {
            if let Some(k) = keep {
                if !k.contains(t.id.as_str()) {
                    continue;
                }
            }
            if !show_completed && t.done && keep.is_none() {
                continue;
            }
            let kids = tree.kids(&t.id);
            let expanded = forced_open || !collapsed.contains(&t.id);
            out.push(Row { id: t.id.clone(), depth, has_kids: !kids.is_empty(), expanded, matched: hits.contains(t.id.as_str()) });
            if expanded {
                walk(tree, kids, depth + 1, collapsed, show_completed, keep, hits, forced_open, view, out);
            }
        }
    }
    walk(tree, &tree.roots, 0, collapsed, show_completed, &keep, &hits, keep.is_some(), view, &mut out);
    out
}

/// Note HTML to the runs the note shows: text with bold, italic and code marks, and paragraph
/// and list item breaks. Enough of the sanitized subset the app writes (p, br, strong/b, em/i,
/// code, ul/ol/li, a).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub link: bool,
}

pub fn note_spans(html: &str) -> Vec<Span> {
    let mut out: Vec<Span> = Vec::new();
    let (mut bold, mut italic, mut code, mut link) = (0, 0, 0, 0);
    let mut i = 0;
    let bytes = html.as_bytes();
    let mut text = String::new();
    let mut blocks = 0;
    let flush = |text: &mut String, out: &mut Vec<Span>, b: i32, it: i32, c: i32, l: i32| {
        if !text.is_empty() {
            out.push(Span { text: decode(text), bold: b > 0, italic: it > 0, code: c > 0, link: l > 0 });
            text.clear();
        }
    };
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let end = html[i..].find('>').map(|e| i + e).unwrap_or(bytes.len() - 1);
            let tag = html[i + 1..end].trim().to_lowercase();
            let closing = tag.starts_with('/');
            let name: String = tag.trim_start_matches('/').chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
            flush(&mut text, &mut out, bold, italic, code, link);
            let delta = if closing { -1 } else { 1 };
            match name.as_str() {
                "strong" | "b" => bold += delta,
                "em" | "i" => italic += delta,
                "code" => code += delta,
                "a" => link += delta,
                "p" | "li" | "div" | "h1" | "h2" | "h3" | "blockquote" if !closing => {
                    if blocks > 0 {
                        text.push('\n');
                    }
                    blocks += 1;
                    if name == "li" {
                        text.push_str("• ");
                    }
                }
                "br" => text.push('\n'),
                _ => {}
            }
            i = end + 1;
        } else {
            let next = html[i..].find('<').map(|n| i + n).unwrap_or(bytes.len());
            text.push_str(&html[i..next]);
            i = next;
        }
    }
    flush(&mut text, &mut out, bold, italic, code, link);
    out
}

fn decode(s: &str) -> String {
    s.replace("&nbsp;", " ").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'").replace("&amp;", "&")
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// The note as the editor edits it: Markdown-style marks for bold, italic and code, a line per
/// paragraph, "- " for list items.
pub fn note_source(html: &str) -> String {
    let mut s = String::new();
    for sp in note_spans(html) {
        let mut t = sp.text.replace("• ", "- ");
        if sp.code {
            t = format!("`{t}`");
        }
        if sp.italic {
            t = format!("*{t}*");
        }
        if sp.bold {
            t = format!("**{t}**");
        }
        s.push_str(&t);
    }
    s
}

/// The editor's text back to the sanitized HTML the store keeps.
pub fn note_html(source: &str) -> String {
    let inline = |line: &str| -> String {
        let mut out = String::new();
        let mut rest = line;
        while !rest.is_empty() {
            if let Some(r) = rest.strip_prefix("**") {
                if let Some(e) = r.find("**") {
                    out.push_str(&format!("<strong>{}</strong>", escape(&r[..e])));
                    rest = &r[e + 2..];
                    continue;
                }
            }
            if let Some(r) = rest.strip_prefix('`') {
                if let Some(e) = r.find('`') {
                    out.push_str(&format!("<code>{}</code>", escape(&r[..e])));
                    rest = &r[e + 1..];
                    continue;
                }
            }
            if let Some(r) = rest.strip_prefix('*') {
                if let Some(e) = r.find('*') {
                    out.push_str(&format!("<em>{}</em>", escape(&r[..e])));
                    rest = &r[e + 1..];
                    continue;
                }
            }
            let ch = rest.chars().next().unwrap();
            out.push_str(&escape(&ch.to_string()));
            rest = &rest[ch.len_utf8()..];
        }
        out
    };
    let mut html = String::new();
    let mut in_list = false;
    for line in source.split('\n') {
        if let Some(item) = line.strip_prefix("- ") {
            if !in_list {
                html.push_str("<ul>");
                in_list = true;
            }
            html.push_str(&format!("<li><p>{}</p></li>", inline(item)));
            continue;
        }
        if in_list {
            html.push_str("</ul>");
            in_list = false;
        }
        html.push_str(&format!("<p>{}</p>", inline(line)));
    }
    if in_list {
        html.push_str("</ul>");
    }
    html
}

/// What the note shows, as one string (paragraphs on lines of their own).
pub fn note_text(html: &str) -> String {
    note_spans(html).into_iter().map(|s| s.text).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes_round_trip() {
        let html = "<p>Draft due <strong>Friday</strong>; keep it to one page.</p>";
        assert_eq!(note_text(html), "Draft due Friday; keep it to one page.");
        assert_eq!(note_source(html), "Draft due **Friday**; keep it to one page.");
        assert_eq!(note_html(&note_source(html)), html);
        assert_eq!(note_text("<p>Hall B, forty seats.</p>"), "Hall B, forty seats.");
        assert_eq!(strip_html(&note_html("Hall B, forty seats. Paid.")), "Hall B, forty seats. Paid.");
    }

    #[test]
    fn relative_times() {
        let now = 1_700_000_000_000;
        assert_eq!(relative(now - 1000, now), "just now");
        assert_eq!(relative(now - 12 * 60_000, now), "12m ago");
        assert_eq!(relative(now - 3 * 3_600_000, now), "3h ago");
        assert_eq!(relative(now - 30 * 3_600_000, now), "yesterday");
        assert_eq!(relative(now - 3 * 86_400_000, now), "3d ago");
        assert_eq!(relative(now - 14 * 86_400_000, now), "2w ago");
    }
}
