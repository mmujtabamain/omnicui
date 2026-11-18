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

## Recording egui for comparison

From the repository root on macOS, using Python 3.10+, Node and Cargo:

```bash
mkdir -p .tmp/taskscape-captures
python3 examples/taskscape/tools/reference-viewer/record-egui.py \
  ~/Desktop/taskscape-references-macos.zip .tmp/taskscape-captures/egui
unzip -q ~/Desktop/taskscape-references-macos.zip 'references/*' references-record.log -d .tmp/taskscape-captures
omnicui compare .tmp/taskscape-captures/references/main-default/shot.png \
  .tmp/taskscape-captures/egui/main-default/shot.png --diff .tmp/main-default-diff.png
(cd .tmp/taskscape-captures && zip -qr ../taskscape-comparison.zip references references-record.log egui)
```

The egui recorder reuses the reference viewer's fixture data and clock in disposable SQLite stores. It reads each viewport size from the ZIP and renders at 1×. Append state IDs to the recorder command to capture a subset. Check `egui/egui-record.log` for scenario failures. The PNGs show the app's content and omit the native macOS window buttons because the renderer runs headlessly.
