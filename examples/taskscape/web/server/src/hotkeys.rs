//! The shortcut catalog as the web app has it: the common crate's catalog and its stored
//! overrides (the same `hotkeys` setting the desktop apps read), with web defaults where a
//! browser keeps the desktop chord for itself.
//!
//! A page never receives ⌘N/Ctrl+N (new window), ⌘⇧N (a private window) or ⌘1–9 (the browser's
//! tabs), so those commands default to the Alt (⌥) chord with the same key here. "Cmd" stays the
//! primary modifier (⌘ on a Mac, Ctrl elsewhere: `common-ui/src/hotkeys.ts`), so every other
//! default is the catalog's own. Screenshots aren't available in a browser, so their global
//! shortcut isn't offered.

use std::collections::BTreeMap;

use anyhow::{anyhow, bail, Result};
use taskscape_common::hotkeys::{Accel, Command, Mods, ResolvedBinding, COMMANDS, HOTKEYS_KEY};
use taskscape_common::Store;

/// Commands a browser can't carry out.
const NOT_ON_THE_WEB: &[&str] = &["screenshot_capture"];

/// The web's default for a command.
pub fn web_default(c: &Command) -> Accel {
    let d = c.default_accel;
    match c.id {
        "new_task" => Accel::new(Mods::ALT, d.key),
        "new_list" => Accel::new(Mods::ALT.with(Mods::SHIFT), d.key),
        id if id.starts_with("switch_list_") => Accel::new(Mods::ALT, d.key),
        _ => d,
    }
}

fn commands() -> impl Iterator<Item = &'static Command> {
    COMMANDS.iter().filter(|c| !NOT_ON_THE_WEB.contains(&c.id))
}

async fn overrides(store: &Store) -> Result<BTreeMap<String, String>> {
    Ok(match store.get_setting(HOTKEYS_KEY).await? {
        Some(json) => serde_json::from_str(&json).unwrap_or_default(),
        None => BTreeMap::new(),
    })
}

/// The catalog with what is in effect: a stored override, else the web default.
pub async fn resolve(store: &Store) -> Result<Vec<ResolvedBinding>> {
    let stored = overrides(store).await?;
    Ok(commands()
        .map(|c| ResolvedBinding {
            id: c.id,
            label: c.label,
            scope: c.scope,
            accel: match stored.get(c.id) {
                Some(s) => s.parse().ok(),
                None => Some(web_default(c)),
            },
            default: web_default(c),
        })
        .collect())
}

/// `hotkeys::set_binding` with the web's defaults: "" unbinds; a chord another command of the
/// same scope has is refused, naming it.
pub async fn set_binding(store: &Store, id: &str, accel: &str) -> Result<()> {
    let command = commands().find(|c| c.id == id).ok_or_else(|| anyhow!("unknown hotkey command: {id}"))?;
    let bound = if accel.is_empty() {
        None
    } else {
        let parsed: Accel = accel.parse().map_err(|_| anyhow!("invalid shortcut: {accel}"))?;
        if !parsed.mods.has_primary() && !parsed.key.is_function() {
            bail!("a shortcut needs Ctrl, Alt or ⌘ (⌃, ⌥)");
        }
        for other in resolve(store).await? {
            if other.id != id && other.scope == command.scope && other.accel == Some(parsed) {
                bail!("already used by “{}”", other.label);
            }
        }
        Some(parsed)
    };
    let mut stored = overrides(store).await?;
    if bound == Some(web_default(command)) {
        stored.remove(id);
    } else {
        stored.insert(id.to_string(), bound.map(|a| a.to_string()).unwrap_or_default());
    }
    store.set_setting(HOTKEYS_KEY, &serde_json::to_string(&stored)?).await
}
