//! Actions and the keymap. The main window's and the capture bar's shortcuts are generated from
//! the app's hotkey catalog (`taskscape_common::hotkeys`, with the user's overrides from
//! Settings); the keys without a modifier (arrows, Enter, Escape) are fixed, as in the app.

use gpui::{actions, App, KeyBinding};
use taskscape_common::hotkeys::{Accel, Key, ResolvedBinding, Scope};

actions!(
    taskscape,
    [
        // The catalog's commands (hotkeys.rs), by id.
        NewTask,
        Search,
        TogglePreview,
        OpenSettings,
        DeleteTask,
        SwitchList1,
        SwitchList2,
        SwitchList3,
        SwitchList4,
        SwitchList5,
        SwitchList6,
        SwitchList7,
        SwitchList8,
        SwitchList9,
        PrevTab,
        NextTab,
        PrevProject,
        NextProject,
        MoveUp,
        MoveDown,
        SelectAll,
        CommandPalette,
        ToggleSplit,
        FocusOtherPane,
        NewList,
        NewProject,
        ToggleCompleted,
        CollapseAll,
        ExpandAll,
        Undo,
        Redo,
        ClearDraft,
        // Fixed keys.
        FocusPrev,
        FocusNext,
        ExtendPrev,
        ExtendNext,
        CollapseOrParent,
        ExpandOrChild,
        Indent,
        Outdent,
        ToggleSelect,
        ToggleDone,
        Rename,
        Escape,
        Confirm,
        Dismiss,
        Activate,
        PaletteUp,
        PaletteDown,
        CaptureSubmit,
        Quit,
        About,
        Minimize,
        Zoom,
    ]
);

/// The keystroke GPUI matches for a catalog accelerator: `Cmd+Shift+N` → `cmd-shift-n`.
pub fn keystroke(a: Accel) -> String {
    let mut s = String::new();
    for (on, m) in [(a.cmd(), "cmd-"), (a.ctrl(), "ctrl-"), (a.alt(), "alt-"), (a.shift(), "shift-")] {
        if on {
            s.push_str(m);
        }
    }
    s.push_str(&key_name(a.key));
    s
}

fn key_name(k: Key) -> String {
    match k {
        Key::Enter => "enter".into(),
        Key::Backspace => "backspace".into(),
        Key::Delete => "delete".into(),
        Key::Space => "space".into(),
        Key::Tab => "tab".into(),
        Key::Escape => "escape".into(),
        Key::Plus => "+".into(),
        Key::ArrowUp => "up".into(),
        Key::ArrowDown => "down".into(),
        Key::ArrowLeft => "left".into(),
        Key::ArrowRight => "right".into(),
        other => other.token().to_lowercase(),
    }
}

fn binding(id: &str, ks: &str) -> Option<KeyBinding> {
    let w = Some("Workspace");
    Some(match id {
        "new_task" => KeyBinding::new(ks, NewTask, w),
        "search" => KeyBinding::new(ks, Search, w),
        "toggle_preview" => KeyBinding::new(ks, TogglePreview, w),
        "open_settings" => KeyBinding::new(ks, OpenSettings, w),
        "delete_task" => KeyBinding::new(ks, DeleteTask, Some("TaskList")),
        "switch_list_1" => KeyBinding::new(ks, SwitchList1, w),
        "switch_list_2" => KeyBinding::new(ks, SwitchList2, w),
        "switch_list_3" => KeyBinding::new(ks, SwitchList3, w),
        "switch_list_4" => KeyBinding::new(ks, SwitchList4, w),
        "switch_list_5" => KeyBinding::new(ks, SwitchList5, w),
        "switch_list_6" => KeyBinding::new(ks, SwitchList6, w),
        "switch_list_7" => KeyBinding::new(ks, SwitchList7, w),
        "switch_list_8" => KeyBinding::new(ks, SwitchList8, w),
        "switch_list_9" => KeyBinding::new(ks, SwitchList9, w),
        "prev_tab" => KeyBinding::new(ks, PrevTab, w),
        "next_tab" => KeyBinding::new(ks, NextTab, w),
        "prev_project" => KeyBinding::new(ks, PrevProject, w),
        "next_project" => KeyBinding::new(ks, NextProject, w),
        "move_up" => KeyBinding::new(ks, MoveUp, Some("TaskList")),
        "move_down" => KeyBinding::new(ks, MoveDown, Some("TaskList")),
        "select_all" => KeyBinding::new(ks, SelectAll, Some("TaskList")),
        "command_palette" => KeyBinding::new(ks, CommandPalette, w),
        "toggle_split" => KeyBinding::new(ks, ToggleSplit, w),
        "focus_other_pane" => KeyBinding::new(ks, FocusOtherPane, w),
        "new_list" => KeyBinding::new(ks, NewList, w),
        "new_project" => KeyBinding::new(ks, NewProject, w),
        "toggle_completed" => KeyBinding::new(ks, ToggleCompleted, w),
        "collapse_all" => KeyBinding::new(ks, CollapseAll, w),
        "expand_all" => KeyBinding::new(ks, ExpandAll, w),
        "undo" => KeyBinding::new(ks, Undo, w),
        "redo" => KeyBinding::new(ks, Redo, w),
        "clear_draft" => KeyBinding::new(ks, ClearDraft, Some("Capture")),
        _ => return None,
    })
}

/// The catalog's bindings for the main window and the capture bar, as resolved from the store.
/// Global ones (the capture bar's summons) belong to the system's global shortcuts.
pub fn catalog_bindings(resolved: &[ResolvedBinding]) -> Vec<KeyBinding> {
    resolved
        .iter()
        .filter(|b| b.scope != Scope::Global)
        .filter_map(|b| b.accel.and_then(|a| binding(b.id, &keystroke(a))))
        .collect()
}

pub fn fixed_bindings() -> Vec<KeyBinding> {
    let l = Some("TaskList");
    vec![
        KeyBinding::new("up", FocusPrev, l),
        KeyBinding::new("down", FocusNext, l),
        KeyBinding::new("shift-up", ExtendPrev, l),
        KeyBinding::new("shift-down", ExtendNext, l),
        KeyBinding::new("left", CollapseOrParent, l),
        KeyBinding::new("right", ExpandOrChild, l),
        KeyBinding::new("tab", Indent, l),
        KeyBinding::new("shift-tab", Outdent, l),
        KeyBinding::new("space", ToggleSelect, l),
        KeyBinding::new("alt-space", ToggleDone, l),
        KeyBinding::new("enter", Rename, l),
        KeyBinding::new("f2", Rename, l),
        KeyBinding::new("escape", Escape, Some("Workspace")),
        KeyBinding::new("enter", Confirm, Some("Sheet")),
        KeyBinding::new("escape", Dismiss, Some("Sheet")),
        KeyBinding::new("escape", Dismiss, Some("Menu")),
        KeyBinding::new("enter", Activate, Some("Button")),
        KeyBinding::new("space", Activate, Some("Button")),
        KeyBinding::new("up", PaletteUp, Some("Palette")),
        KeyBinding::new("down", PaletteDown, Some("Palette")),
        KeyBinding::new("escape", Dismiss, Some("Palette")),
        KeyBinding::new("cmd-enter", CaptureSubmit, Some("Capture")),
        KeyBinding::new("escape", Dismiss, Some("Capture")),
        KeyBinding::new("up", PaletteUp, Some("Settings")),
        KeyBinding::new("down", PaletteDown, Some("Settings")),
        KeyBinding::new("escape", Dismiss, Some("Settings")),
        KeyBinding::new("cmd-q", Quit, None),
    ]
}

/// The catalog's glyphs for a command's current binding (`⌘⇧N`), or "" when it is unbound.
pub fn glyphs(resolved: &[ResolvedBinding], id: &str) -> String {
    resolved.iter().find(|b| b.id == id).and_then(|b| b.accel).map(|a| a.glyphs()).unwrap_or_default()
}

pub fn rebind(cx: &mut App, resolved: &[ResolvedBinding]) {
    cx.clear_key_bindings();
    cx.bind_keys(crate::text_field::bindings());
    cx.bind_keys(fixed_bindings());
    cx.bind_keys(catalog_bindings(resolved));
}
