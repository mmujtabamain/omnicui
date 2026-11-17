//! The registry: the widget library's own (taskscape_ui::widgets, found from its files), and later
//! the app's screens.
use serde_json::Value;
pub use taskscape_ui::Mounted;

/// contract id -> a fresh component, built from the test's props.
pub fn create(component: &str, props: Value) -> Result<Box<dyn Mounted>, String> {
    if let Some(made) = taskscape_ui::widgets::mount(component, props) {
        return made.map_err(|e| format!("props for {component}: {e}"));
    }
    Err(format!("no component {component}"))
}
