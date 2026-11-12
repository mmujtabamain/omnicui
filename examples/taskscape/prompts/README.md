# Prompts that build the Taskscape example

Taskscape is a macOS task manager. It was built as Tauri v2 apps (React 19 and Tailwind v4 in a webview, Rust behind), and is ported here to a native GPUI app (Zed's Rust UI framework) and to a web app, with omnicui checking that the three behave the same and keep one design.

Three agents build it, each from one of these prompts:

| Prompt | Runs | Builds |
|---|---|---|
| [P0.md](P0.md) | first | the omnicui project: contracts, fixtures, the product spec, the design notes, the `tauri` target |
| [P1.md](P1.md) | after P0, alongside P2 | the GPUI port and its target |
| [P2.md](P2.md) | after P0, alongside P1 | the web port, its server and its target |

Each prompt here is what to build. The environment an agent runs in adds its own instructions on how to work there: how to report progress, share screenshots and use the machine's toolchains. Those are kept with that environment, not here. The agents' transcripts, and a summary of each, are in [../process/](../process/) once they have run.
