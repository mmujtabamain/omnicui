//! `taskscape-fixture <seed> [<folder>]`: loads one fixture into a fresh data folder under
//! `<folder>` (default: the system's temporary folder) and prints, as one line of JSON, the
//! home folder it made (`HOME` for the app) and the fixture's keys mapped to the store's ids.

use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let Some(seed) = args.next() else {
        eprintln!(
            "usage: taskscape-fixture <seed> [<folder>]; the seeds are {}",
            taskscape_fixtures::seed_names().join(", ")
        );
        std::process::exit(2);
    };
    let root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("taskscape-fixtures"));
    match taskscape_fixtures::load(&seed, &root).await {
        Ok(loaded) => println!(
            "{}",
            serde_json::json!({ "home": loaded.home, "keys": loaded.keys })
        ),
        Err(e) => {
            eprintln!("taskscape-fixture: {e:#}");
            std::process::exit(1);
        }
    }
}
