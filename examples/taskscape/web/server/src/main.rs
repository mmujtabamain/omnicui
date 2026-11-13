//! Taskscape for the web: one process that serves the built frontend and answers it over HTTP
//! with `taskscape_common`'s `Store`, the same storage the desktop apps use.
//!
//! The API is the common crate's axum `data_router` (lists and tasks), plus the routes the app's
//! commands need that it lacks (projects, the Trash, notes, uploads, settings, shortcuts), all
//! under `/api`. Everything else is the frontend: a file from the build folder, or `index.html`
//! for a deep link (`/lists/<id>/tasks/<id>`), so the browser's own URLs and history work.
//!
//! ```text
//! taskscape-web --dist <built frontend> [--host 127.0.0.1] [--port 7430]
//! ```
//!
//! The data is `~/.taskscape/` (the store finds it from `HOME`). Once it is listening the server
//! prints one line, `listening <port>`, on standard output; everything else goes to standard
//! error. Built with the `test-hooks` feature, `--test-hooks <folder>` starts it with no data and
//! adds the fixture loader and the event log that the omnicui target drives (hooks.rs).

mod alerts;
mod api;
mod hotkeys;
mod lists;
mod live;
mod static_files;

#[cfg(feature = "test-hooks")]
mod hooks;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use axum::{routing::any, Router};

use live::Server;

struct Args {
    dist: PathBuf,
    host: String,
    port: u16,
    test_hooks: Option<PathBuf>,
}

fn parse_args() -> Result<Args> {
    let mut args = Args { dist: PathBuf::new(), host: "127.0.0.1".into(), port: 7430, test_hooks: None };
    let mut it = std::env::args().skip(1);
    let mut dist = None;
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().with_context(|| format!("{name} needs a value"));
        match a.as_str() {
            "--dist" => dist = Some(PathBuf::from(value("--dist")?)),
            "--host" => args.host = value("--host")?,
            "--port" => args.port = value("--port")?.parse().context("--port")?,
            "--test-hooks" => args.test_hooks = Some(PathBuf::from(value("--test-hooks")?)),
            "-h" | "--help" => {
                eprintln!("usage: taskscape-web --dist <folder> [--host 127.0.0.1] [--port 7430]");
                std::process::exit(0);
            }
            other => bail!("unknown argument {other} (try --help)"),
        }
    }
    args.dist = dist.context("--dist <the built frontend> is required")?;
    if !args.dist.join("index.html").is_file() {
        bail!("{} has no index.html: build the frontend first (web/scripts/build.sh)", args.dist.display());
    }
    Ok(args)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args()?;
    let server = Arc::new(Server::new(args.dist.clone()));

    let mut router = Router::new().route("/api/{*rest}", any(live::forward));
    match &args.test_hooks {
        #[cfg(feature = "test-hooks")]
        Some(data) => router = router.merge(hooks::routes(data.clone())),
        #[cfg(not(feature = "test-hooks"))]
        Some(_) => bail!("--test-hooks needs a build with the test-hooks feature"),
        None => {
            let store = taskscape_common::Store::open().await.context("opening ~/.taskscape")?;
            server.install(live::Live::new(Arc::new(store), None)).await;
        }
    }
    let router = router.fallback(static_files::serve).with_state(server);

    let listener = tokio::net::TcpListener::bind((args.host.as_str(), args.port)).await?;
    let port = listener.local_addr()?.port();
    // The one line on standard output: a launcher (or the omnicui target) waits for it.
    println!("listening {port}");
    eprintln!("[taskscape-web] serving {} on http://{}:{port}", args.dist.display(), args.host);
    axum::serve(listener, router).await?;
    Ok(())
}
