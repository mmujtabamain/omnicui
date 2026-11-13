//! Taskscape's main window, native in GPUI. `taskscape-gpui --capture` shows the capture bar
//! instead (the menu bar agent's job in the Tauri app).

use std::rc::Rc;

use gpui::{
    point, prelude::*, px, size, App, Application, Bounds, Menu, MenuItem, TitlebarOptions, WindowBackgroundAppearance, WindowBounds,
    WindowDecorations, WindowKind, WindowOptions,
};
use taskscape_gpui::{capture::CaptureBar, keys, Assets, Db, Surface, Workspace};

fn main() {
    let capture_only = std::env::args().any(|a| a == "--capture");
    let db = match Db::open() {
        Ok(db) => Rc::new(db),
        Err(e) => {
            eprintln!("taskscape: can't open the store: {e:#}");
            std::process::exit(1);
        }
    };
    Application::new().with_assets(Assets).run(move |cx: &mut App| {
        taskscape_gpui::init(cx);
        keys::rebind(cx, &db.hotkeys());
        cx.on_action(|_: &keys::Quit, cx| cx.quit());
        cx.set_menus(vec![
            Menu { name: "Taskscape".into(), items: vec![MenuItem::action("Settings…", keys::OpenSettings), MenuItem::separator(), MenuItem::action("Quit Taskscape", keys::Quit)] },
            Menu {
                name: "File".into(),
                items: vec![MenuItem::action("New Task", keys::NewTask), MenuItem::action("New List…", keys::NewList), MenuItem::action("New Project…", keys::NewProject)],
            },
            Menu { name: "Edit".into(), items: vec![MenuItem::action("Undo", keys::Undo), MenuItem::action("Redo", keys::Redo), MenuItem::separator(), MenuItem::action("Select All Tasks", keys::SelectAll)] },
            Menu {
                name: "View".into(),
                items: vec![
                    MenuItem::action("Command Palette…", keys::CommandPalette),
                    MenuItem::action("Toggle Preview", keys::TogglePreview),
                    MenuItem::action("Toggle Split View", keys::ToggleSplit),
                    MenuItem::action("Show / Hide Completed", keys::ToggleCompleted),
                ],
            },
        ]);
        let dark = matches!(cx.window_appearance(), gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark);
        if capture_only {
            let bounds = Bounds::centered(None, size(px(480.), px(200.)), cx);
            let db = db.clone();
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: None,
                    kind: WindowKind::PopUp,
                    focus: true,
                    show: true,
                    window_background: WindowBackgroundAppearance::Transparent,
                    ..Default::default()
                },
                move |window, cx| {
                    cx.new(|cx| {
                        let mut bar = CaptureBar::new(db, dark, window, cx);
                        bar.on_dismiss = Some(Rc::new(|window, _| window.minimize_window()));
                        bar
                    })
                },
            )
            .ok();
            cx.activate(true);
            return;
        }
        let bounds = Bounds { origin: point(px(120.), px(80.)), size: size(px(1180.), px(760.)) };
        let db2 = db.clone();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions { title: Some("Taskscape".into()), appears_transparent: true, traffic_light_position: Some(point(px(18.), px(16.))) }),
                window_decorations: Some(WindowDecorations::Client),
                window_min_size: Some(size(px(880.), px(560.))),
                focus: true,
                show: true,
                ..Default::default()
            },
            move |window, cx| {
                let ws = cx.new(|cx| Workspace::new(db2, Surface::Window, dark, window, cx));
                let f = ws.read(cx).root_focus.clone();
                window.focus(&f);
                ws
            },
        )
        .ok();
        cx.activate(true);
    });
}
