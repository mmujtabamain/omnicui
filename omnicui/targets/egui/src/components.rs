//! The registry: which of the app's components answers to which contract id, and how its props
//! and events become the contract's. One entry per component.
use desktop::counter::Counter;
use desktop::switch::Switch;
use desktop::Look;
use serde_json::Value;

/// A mounted component, as the adapter drives it.
pub trait Mounted {
    /// Draws one frame. `emit` reports an event with its contract name and fields.
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, Value));
}

impl Mounted for Counter {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, Value)) {
        if let Some(e) = self.ui(ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

impl Mounted for Switch {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, Value)) {
        if let Some(e) = self.ui(ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// contract id → a fresh component, built from the test's props.
pub fn create(component: &str, props: Value) -> Result<Box<dyn Mounted>, String> {
    let bad = |e: serde_json::Error| format!("props for {component}: {e}");
    match component {
        "ui.counter" => Ok(Box::new(Counter::new(serde_json::from_value(props).map_err(bad)?))),
        "ui.switch" => Ok(Box::new(Switch::new(serde_json::from_value(props).map_err(bad)?))),
        other => Err(format!("no component {other}")),
    }
}
