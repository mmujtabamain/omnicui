# Taskscape: a real app, checked by omnicui across three ports

Taskscape is a macOS task manager. It has a full task window (projects, lists, nested tasks, a rich-text inspector, attachments, a command palette, settings) and a menu-bar capture bar. It was built as two Tauri v2 apps. Here it is the reference for two ports:
- a native **GPUI** app (Zed's Rust UI framework);
- a **web** app.

omnicui checks that all three behave the same and keep one design, each idiomatic for its platform. This is the step from the proof-of-concept examples to a complicated, real-world app.

| Folder | What it is | Status |
|---|---|---|
| `tauri/` | The Tauri app, copied as-is from its own repository at commit `09f3f4c` (its tracked files only). The reference. | vendored |
| `prompts/` | What each of the three building agents was asked to build (P0, P1, P2) | written |
| `omnicui/` | The omnicui project: contracts, fixtures, product spec, design notes, the `tauri` target | P0 |
| `gpui/` | The GPUI port and its target | P1 |
| `web/` | The web port, its server and its target | P2 |
| `process/` | The building agents' transcripts and a summary of each | after the build |

The app's own documentation is in `tauri/AGENTS.md` and `tauri/.llm/`.
