# The desktop widget library (egui): conventions

The same widgets as the web's `packages/ui`, held to the same contracts (`omnicui/contracts/<name>.contract.yaml`), drawn with egui 0.37. omnicui's desktop target mounts each widget headlessly and reads it through AccessKit.

## One widget, one file

- A widget is `src/widgets/<contract name>.rs` (`checkbox.rs`). While it is being built it lives in `drafts/<contract name>.rs`. You only ever write your own draft, and never edit another file.
- Its first line names its contract, in this form exactly:
  ```rust
  //! contract: omnicui/contracts/checkbox.contract.yaml (id: ui.checkbox). <one line on what it is>
  ```
  build.rs finds widgets by this line and registers them.
- `src/widgets/checkbox.rs` is a finished widget. Read it first, and follow it.

## The shape of a widget

```rust
use crate::omnicui::checkbox::{parts, CheckboxEvent, CheckboxProps};   // generated from the contract
use crate::{part, rem, text, icon, Look};

pub struct Checkbox { p: CheckboxProps, /* state */ }

impl Checkbox {
    pub fn new(p: CheckboxProps) -> Self { … }
    /// One frame: draws the widget and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<CheckboxEvent> { … }
}

crate::mounted!(Checkbox, CheckboxProps);   // last line: lets the target mount it
```

- **Props** are the generated `<Name>Props`, with the contract's defaults. A prop named with a Rust keyword is raw: `p.r#type`.
- **Events** are the generated `<Name>Event` variants. Return them from `ui()`; never print or store them elsewhere.
- **State:** a test mounts the widget with props, then presses and types. Keep the state the presses change on the struct, the way a checkbox keeps `checked`.

## Parts: what a contract can see

Every part the contract lists is a node in AccessKit, found by its `author_id`, which is the part id (`ui.checkbox.control`). Set it with `part(ui, &response, parts::CONTROL)` on the part's `Response`.

- A part with no widget of its own (an indicator, a label region) still needs a node. Use `ui.interact(rect, id, Sense::hover())`, give it `widget_info`, and tag it with `part`.
- **The fields a test reads come from that node's WidgetInfo:**
  - `role`: `Role::Button`, `Role::CheckBox`, `Role::Switch`, `Role::Slider`, `Role::TextInput`, `Role::Heading`, `Role::Status`, `Role::GenericContainer`.
  - `name`: the label. It is what a screen reader announces, for example an icon button's `ariaLabel`.
  - `text`: the node's value, or else its label.
  - `checked`: `WidgetInfo::selected(Role::CheckBox, enabled, checked, label)`.
  - `selected`: a chosen tab, segment or option.
  - `disabled`, `focused`.
- **A part that is not shown is not drawn at all.** A closed menu or an unchecked mark has no node, and the target reports it as 0 × 0, which is "not visible".
- Text a test reads with `text:` must be the node's value or label exactly as the contract writes it.

## Look

- **Colours:** only `look.t.*`, the web's tokens converted (`surface_1`, `content_2`, `accent_500`, `edge_2`, `wash_1`, `focus_1`, `danger_500`…). Dark mode is the same names. Never write a hex colour.
- **Sizes:** in the web's rem, where `rem(1.0)` = 14 px. Port the web widget's Tailwind sizes: `size-[1.143rem]` is `rem(1.143)`, `px-2` is 8 px, `gap-space-4` is 6 px (spacing tokens 1–8 are 2, 2.6, 3.5, 5.25, 7, 8.75, 12.25 and 17.5 px).
- **Text:** `text("Label", 13.0, 500)`, which is Inter at a px size and weight (400, 500, 600 or 700).
- **Icons:** `icon("check", 15.0)` by Material Symbols name. The names are in `src/icons.rs`.
- **Radii:** field 4 px (5 px is the web's `--radius-field`), control 7 px, panel 10.5 px, full for round.
- **Focus ring:** 2 px of `focus_1` outside the widget while it has keyboard focus.
- **Hover:** a `wash_1` or `wash_2` fill.

## Behaviour

- **Keys:** a widget a contract presses keys on must take focus. Use `Sense::click()` (or `focusable_noninteractive`) and `response.request_focus()` on a press where the web focuses it.
  - Read keys with `ui.input(|i| i.key_pressed(Key::Space))` while `response.has_focus()`.
  - egui turns Enter and Space on a focused button into a click. Where the contract says a key must not act, check for it.
- **The platform modifier** is `i.modifiers.command`: Command on macOS, Control elsewhere.
- **Text fields** use `egui::TextEdit`. Tag the `Response` it returns, and use `.hint_text()` for a placeholder.
- **Menus, popovers, tooltips and dialogs** are egui `Area`s or popups inside the window. **Never draw a scrim or a backdrop:** on the desktop those tests are tagged web-only (decision 2 and the platform rubric).
- **Animation:** use egui's `ctx.animate_bool_with_time` or `animate_value_with_time`, with the durations the web uses (0.18 s fast, 0.28 s base). Tests settle when nothing is animating.

## How to work

1. **Read** the contract, the widget's spec (`spec/library/widgets/<Name>.json`), the web widget you are porting (`packages/ui/src/components/ui/<kebab-name>.tsx`), `src/lib.rs`, `src/widgets/checkbox.rs`, and the generated `src/omnicui/<name>.rs`.
2. **Write** `drafts/<name>.rs`.
3. **Run** `python3 apps/desktop/scripts/check-widget.py <name> --slot <your slot>` from `examples/taskscape`. It builds your draft with the finished widgets and runs the contract. A test tagged `only: [web]` shows "not applicable"; ignore it.
4. **Fix** until it exits 0, then look at two or three of its screenshots (the path is printed) and fix what looks wrong next to the web widget.

Do not add dependencies, edit `Cargo.toml`, the generated types, or anything outside your draft.
