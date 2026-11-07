//! The desktop app's window: the counter and the switch, a dark colour scheme toggle, and the
//! events the components report. `cargo run --release` (or `run.sh app desktop`).
//! OMNICUI_DEFECTS=… switches on the components' deliberate defects.
use desktop::counter::Counter;
use desktop::switch::Switch;
use desktop::{fonts, Look};
use eframe::egui;

struct App {
    dark: bool,
    counter: Counter,
    switch: Switch,
    events: Vec<String>,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let look = Look::of(if self.dark { "dark" } else { "light" });
        ui.ctx().set_theme(if self.dark { egui::Theme::Dark } else { egui::Theme::Light });
        egui::Frame::NONE.fill(look.bg).inner_margin(24.0).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.label(egui::RichText::new("Counter").small().color(look.off));
            if let Some(e) = self.counter.ui(ui, look) {
                self.events.insert(0, format!("{e:?}"));
            }
            ui.add_space(24.0);
            ui.label(egui::RichText::new("Switch").small().color(look.off));
            if let Some(e) = self.switch.ui(ui, look) {
                self.events.insert(0, format!("{e:?}"));
            }
            ui.add_space(32.0);
            ui.separator();
            ui.checkbox(&mut self.dark, egui::RichText::new("Dark colour scheme").color(look.fg));
            ui.label(egui::RichText::new("Events (newest first):").color(look.fg));
            for e in self.events.iter().take(8) {
                ui.monospace(egui::RichText::new(e).color(look.fg));
            }
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("Desktop app (egui)").with_inner_size([420.0, 420.0]),
        ..Default::default()
    };
    eprintln!("Opening the window (close it to stop).");
    eframe::run_native(
        "desktop",
        options,
        Box::new(|cc| {
            fonts(&cc.egui_ctx);
            Ok(Box::new(App {
                dark: false,
                counter: Counter::new(Default::default()),
                switch: Switch::new(Default::default()),
                events: vec![],
            }))
        }),
    )
}
