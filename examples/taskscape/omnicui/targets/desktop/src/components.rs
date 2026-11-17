//! The registry: the widget library's widgets (taskscape_ui::widgets) and the app's screens
//! (taskscape_app::screens), each found from its files.
use serde_json::Value;
pub use taskscape_ui::Mounted;

/// contract id -> a fresh component, built from the test's props.
pub fn create(component: &str, props: Value) -> Result<Box<dyn Mounted>, String> {
    if let Some(made) = taskscape_app::mount(component, props.clone()) {
        return made.map_err(|e| format!("{component}: {e}"));
    }
    if let Some(made) = taskscape_ui::widgets::mount(component, props) {
        return made.map_err(|e| format!("props for {component}: {e}"));
    }
    Err(format!("no component {component}"))
}
