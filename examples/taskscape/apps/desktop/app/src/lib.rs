//! The Taskscape desktop app (egui): its data on the reference's storage, what the user is looking
//! at, and its screens. A screen is `src/screens/<contract name>.rs`, held to
//! omnicui/contracts/<name>.contract.yaml; the main window and the other windows compose the same
//! screens the contracts check one at a time.
pub mod app;
pub mod data;
pub mod state;

pub use app::App;

/// The screens (src/screens/), found by build.rs, and the registry from contract id to screen.
pub mod screens {
    include!(concat!(env!("OUT_DIR"), "/screens.rs"));
}

/// Mounts a screen for a test: a fresh store loaded with the props' `dataset` (standard by default).
pub fn mount(id: &str, props: serde_json::Value) -> Option<Result<Box<dyn taskscape_ui::Mounted>, String>> {
    if !id.starts_with("app.") {
        return None;
    }
    screens::sheets::reset_roll();
    let dataset = props.get("dataset").and_then(|d| d.as_str()).unwrap_or("standard").to_owned();
    let app = match App::demo(&dataset) {
        Ok(a) => a,
        Err(e) => return Some(Err(format!("demo data {dataset}: {e}"))),
    };
    screens::mount(id, app, props)
}
