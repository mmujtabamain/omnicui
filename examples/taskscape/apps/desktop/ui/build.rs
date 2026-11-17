//! Finds the widgets: every src/widgets/<name>.rs whose first line names its contract
//! (`//! contract: … (id: ui.<name>)`) becomes a module, and the registry maps the contract id to its
//! `mount` function. Adding a widget is adding its file; nothing else is edited.
use std::fmt::Write;

fn main() {
    let dir = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/widgets");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut found = vec![];
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let path = e.path();
            println!("cargo:rerun-if-changed={}", path.display());
            if path.extension().and_then(|x| x.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            let first = text.lines().next().unwrap_or_default();
            let Some(at) = first.find("(id: ") else { continue };
            let id = first[at + 5..].split(')').next().unwrap_or_default().trim().to_owned();
            let module = path.file_stem().unwrap().to_string_lossy().into_owned();
            found.push((module, id, path));
        }
    }
    found.sort();
    let mut out = String::new();
    for (module, _, path) in &found {
        writeln!(out, "#[path = {:?}]\npub mod {module};", path.display().to_string()).unwrap();
    }
    writeln!(out, "\n/// contract id -> its widget, built from a test's props.\npub fn mount(id: &str, props: serde_json::Value) -> Option<Result<Box<dyn crate::Mounted>, String>> {{\n    match id {{").unwrap();
    for (module, id, _) in &found {
        writeln!(out, "        {id:?} => Some({module}::mount(props)),").unwrap();
    }
    writeln!(out, "        _ => None,\n    }}\n}}\n\n/// The contract ids this build has a widget for.\npub const IDS: &[&str] = &[{}];", found.iter().map(|(_, id, _)| format!("{id:?}")).collect::<Vec<_>>().join(", ")).unwrap();
    std::fs::write(std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("widgets.rs"), out).unwrap();
}
