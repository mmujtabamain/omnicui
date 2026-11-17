//! Finds the screens: every src/screens/<name>.rs whose first line names its contract
//! (`//! contract: … (id: app.<name>)`) becomes a module, and the registry maps the contract id to
//! its `mount(app, props)` function. Adding a screen is adding its file.
use std::fmt::Write;

fn main() {
    let dir = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/screens");
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
            let module = path.file_stem().unwrap().to_string_lossy().into_owned();
            // A screen file names its contract; a shared helper file (no contract) is a plain module.
            let id = first.find("(id: ").map(|at| first[at + 5..].split(')').next().unwrap_or_default().trim().to_owned());
            found.push((module, id, path));
        }
    }
    found.sort();
    let mut out = String::new();
    for (module, _, path) in &found {
        writeln!(out, "#[path = {:?}]\npub mod {module};", path.display().to_string()).unwrap();
    }
    writeln!(out, "\n/// contract id -> its screen, mounted on the app from a test's props.\npub fn mount(id: &str, app: crate::App, props: serde_json::Value) -> Option<Result<Box<dyn taskscape_ui::Mounted>, String>> {{\n    match id {{").unwrap();
    for (module, id, _) in &found {
        if let Some(id) = id {
            writeln!(out, "        {id:?} => Some({module}::mount(app, props)),").unwrap();
        }
    }
    writeln!(out, "        _ => None,\n    }}\n}}").unwrap();
    std::fs::write(std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("screens.rs"), out).unwrap();
}
