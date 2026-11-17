//! The OS's file dialogs, in the app binary only; screen tests have none (the calls return None).

/// Where to save a list's JSON export, starting from `default_name`.
pub fn save_json(default_name: &str) -> Option<std::path::PathBuf> {
    #[cfg(feature = "window")]
    {
        rfd::FileDialog::new().set_file_name(default_name).add_filter("JSON", &["json"]).save_file()
    }
    #[cfg(not(feature = "window"))]
    {
        let _ = default_name;
        None
    }
}

/// A list export to import.
pub fn open_json() -> Option<std::path::PathBuf> {
    #[cfg(feature = "window")]
    {
        rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file()
    }
    #[cfg(not(feature = "window"))]
    {
        None
    }
}
