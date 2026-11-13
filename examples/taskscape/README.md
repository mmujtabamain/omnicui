# Taskscape

A task manager rebuilt from a reference desktop app: a web version first (Django REST backend, React frontend, its own widget package), a desktop version later. Work is staged; this folder grows as each stage lands.

| Folder | What is in it |
|---|---|
| `spec/` | What the app does, taken from the reference: the feature list, and (regenerated, not committed) the recorded screen states in `spec/references/` |
| `tools/reference-viewer/` | Runs the reference's own frontend in Chromium on fixture data and records each screen state |
| `process/` | The agent definitions, workflow scripts and per-run records used to build this |

## Recording the reference

```bash
cd tools/reference-viewer
npm install
TASKSCAPE_REFERENCE=/path/to/reference node prepare.mjs   # copies its frontend to ~/.cache; the reference is only read
node serve.mjs &                                          # its own Vite dev servers: main on 1420, capture bar on 1421
node capture.mjs                                          # every state in states.mjs -> spec/references/<state>/
node contact-sheet.mjs /tmp/sheet                         # thumbnails, 24 states per image
```

Each state folder holds `shot.png`, `tree.txt` (React's component tree: names, props, boxes), `elements.json` (roles, labels, colours, type) and `meta.json` (the commands the app sent).
