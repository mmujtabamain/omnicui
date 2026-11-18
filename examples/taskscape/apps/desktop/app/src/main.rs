//! The Taskscape desktop app: the main window, and the Trash, Settings and confirmation windows it opens.
use eframe::egui;
use taskscape_app::data::Data;
use taskscape_app::screens::mainwindow::MainWindow;
use taskscape_app::App;
use taskscape_ui::Look;

struct Taskscape {
    app: App,
    window: MainWindow,
    menu: Option<taskscape_app::menu::NativeMenu>,
    /// The Global hotkeys (Capture bar, Screenshot capture); None where the OS offers none.
    hotkeys: Option<taskscape_app::hotkeys::GlobalHotkeys>,
    capture: taskscape_app::capture_window::CaptureWindow,
}

impl eframe::App for Taskscape {
    /// Every window clears to transparent: the panels paint their own fill, and the capture bar's rounded card
    /// sits on nothing (eframe's default is a dark grey, which macOS showed as a black box).
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // The Global hotkeys: registered from Settings (again when one changes), and what was pressed.
        if let Some(hotkeys) = self.hotkeys.as_mut() {
            hotkeys.sync(&self.app);
            for action in hotkeys.pressed() {
                match action {
                    taskscape_app::hotkeys::GlobalAction::ToggleCaptureBar => self.capture.toggle(&mut self.app, &ctx),
                    taskscape_app::hotkeys::GlobalAction::ScreenshotCapture => self.capture.screenshot(&mut self.app, &ctx),
                }
            }
        }
        // The OS menu's List items.
        if let Some(action) = self.menu.as_ref().and_then(|m| m.poll()) {
            match action {
                taskscape_app::menu::MenuAction::ImportList => self.app.import_list(None),
                taskscape_app::menu::MenuAction::ExportList => {
                    if let Some(list) = self.app.active_list() {
                        self.app.export_list(&list);
                    }
                }
            }
        }
        // The theme: the saved one, or the system's while it says System.
        let dark = match self.app.data.settings.theme.as_str() {
            "dark" => true,
            "light" => false,
            _ => ctx.system_theme().is_some_and(|t| t == egui::Theme::Dark),
        };
        let look = Look::of(if dark { "dark" } else { "light" });
        ctx.set_theme(if dark { egui::Theme::Dark } else { egui::Theme::Light });
        // Reduced motion: On cuts egui's own animations; System follows the OS (egui has no signal for it).
        let reduced = self.app.data.settings.reduced_motion == "on";
        ctx.all_styles_mut(|s| s.animation_time = if reduced { 0.0 } else { 1.0 / 12.0 });
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(look.t.surface_1)).show(ui, |ui| {
            self.window.show(&mut self.app, ui, look);
        });
        self.window.windows(&mut self.app, &ctx, look);
        self.capture.show(&mut self.app, &ctx, look);
        #[cfg(target_os = "macos")]
        taskscape_app::chrome::align_window_controls(ctx.zoom_factor());
    }
}

fn main() -> eframe::Result {
    let data = match Data::open() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("taskscape: could not open the data folder ({}): {e:#}", taskscape_storage::paths::root_dir().display());
            std::process::exit(1);
        }
    };
    let options = eframe::NativeOptions {
        viewport: taskscape_app::chrome::main_window(egui::ViewportBuilder::default().with_title(taskscape_app::chrome::MAIN_TITLE).with_inner_size([1180.0, 760.0]).with_min_inner_size([820.0, 560.0]).with_app_id("taskscape")),
        ..Default::default()
    };
    eframe::run_native(
        taskscape_app::chrome::MAIN_TITLE,
        options,
        Box::new(|cc| {
            taskscape_ui::fonts(&cc.egui_ctx);
            #[cfg(target_os = "macos")]
            taskscape_app::chrome::not_movable_by_os(cc);
            let mut window = MainWindow::new();
            window.in_place = false;
            let mut app = App::new(data);
            app.native_window = true;
            let menu = taskscape_app::menu::NativeMenu::install();
            let hotkeys = taskscape_app::hotkeys::GlobalHotkeys::new(&cc.egui_ctx);
            Ok(Box::new(Taskscape { app, window, menu, hotkeys, capture: taskscape_app::capture_window::CaptureWindow::new() }))
        }),
    )
}
