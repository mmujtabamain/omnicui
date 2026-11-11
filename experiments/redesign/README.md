# Experiment: behaviour when the UI is different

Does omnicui check behaviour when two versions of a component look nothing alike?

To find out, the web app got a second design of the counter and the switch, in `apps/web/src/components/redesign/`:
- **the counter** is a vertical card, with the value in a coloured pill and a round icon button;
- **the switch** is a native checkbox with a tick: no track, no thumb, no animation.

The second design has the same props, the same events and the same part tags as the first. It runs here as the target `webalt`, next to `web` (a link to the reference web target, the original design), against the reference contracts (also a link), on omnicui 0.2.2.

Variants then change one thing each, switched on with `OMNICUI_DEFECTS`.
- **Look only (the behaviour is the same):**
  - `copy`: the value reads "3 taps";
  - `srlabel`: a visually-hidden "Count: " sits in front of the value;
  - `toolbar`: a heading and a Reset button that the contract doesn't name;
  - `pill`: a switch whose thumb slides up, over 400 ms, linearly.
- **Broken behaviour:**
  - `double`: each press adds two;
  - `divbutton`: a `<div role="button">` with no key handling;
  - `noevent`: no `toggled` event;
  - `nospace`: Space doesn't toggle.

The rules: omnicui and the contracts are not changed, and no part is faked to get a pass.

```sh
./run.sh setup web
experiments/redesign/run.sh            # every run below (about 2 minutes); output in runs/<run>/
experiments/redesign/run.sh base       # one of them
```

## Results (webalt; `web` passes everything in every run)

| Run | webalt | Verdict | What omnicui said |
|---|---|---|---|
| base (card + checkbox) | 9 pass, 1 fail; 5 pixel warnings | **false failure** | `a press turns it on, and the thumb slides across`: `trace ui.switch.thumb: no part` (and a hint to look the part up at each frame, which is wrong here: a checkbox has no thumb). Every screenshot pair warns, area 92–100%. |
| counter.copy | 3 fail | **false failure** (strictly, the contract asked for "3") | `value.text: expected "3", got "3 taps"` |
| counter.srlabel | 3 fail | **false failure** | `value.text: expected "5", got "Count:\n5"`: the web adapter reads `innerText`, which includes visually-hidden text |
| counter.toolbar | pass | right | parts the contract doesn't name are ignored |
| switch.pill | pass; timing and motion warnings | right, but incomplete | `trace thumb web ↔ webalt 99 ms`, `clip 134 ms`; that the thumb moves up instead of across isn't noticed |
| counter.double | 2 fail | right | `value.text: expected "3", got "6"`, `changed.last.count: expected 3, got 6`; the generated keyboard tests pass (key and press both add two) |
| counter.divbutton | 3 fail | right | `Enter on increment didn't do what a press does: … the key gave value.text "0"` |
| switch.noevent | 2 fail | right, but tangled | `toggled: expected 1 time, got 0 times`, in the same failing test as the thumb's false failure |
| switch.nospace | 3 fail | right | `Space on switch didn't do what a press does` |
| fail-on-pixels | exit 1 | a gap | 2 pixel warnings fail the run: nothing says that webalt is meant to look different |

## Contract experiments (`contract-variants/`)

| Switch contract | webalt | What it shows |
|---|---|---|
| a: `thumb: { optional: true }` | still fails | `optional` covers masks and crops only, not traces |
| b: the trace and the clip in a test of their own | that test fails | the failure moves, it doesn't go away |
| c: no trace, no clip | pass | but the original designs lose their motion check (and `part thumb is never used`) |
| d: `record: { part: switch }`, no trace | pass; motion warning 78 ms (the pill: 134 ms) | what works today: a clip needs no part inside the switch |

## What omnicui is missing

1. **A part that only some designs have.** `optional: true` should cover traces, clips and expectations too: such a test should be "not applicable" where the target has no such part, instead of failing. A part that is missing and isn't optional should say `optional: true` is the way out, not give the per-frame hint.
2. **Designs.** `design:` in `target.yaml`, so that screenshots are compared only between targets of the same design, while behaviour, keyboard and motion are still checked everywhere. Without it, `--fail-on pixels` can't be used between two designs.
3. **Looser text.** `text: { contains: "3" }` or `matches:`. And a decision on whether `text` means what is shown, or what a screen reader reads (`innerText` gives the second).
4. **Direction.** A trace compares only the timing along the movement, so the direction and the distance aren't compared.
