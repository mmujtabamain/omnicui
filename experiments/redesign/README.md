# Experiment: behaviour when the UI is different

Does omnicui check behaviour when two versions of a component look nothing alike?

To find out, the web app got a second design of the counter and the switch, in `apps/web/src/components/redesign/`:
- **the counter** is a vertical card, with the value in a coloured pill and a round icon button;
- **the switch** is a native checkbox with a tick: no track, no thumb, no animation.

The second design has the same props, the same events and the same part tags as the first. It runs here as the target `webalt`, next to `web` (a link to the reference web target, the original design), against the reference contracts (also a link), first on omnicui 0.2.2, and again on 0.2.3, which was built from what 0.2.2 got wrong here. webalt's target.yaml now says `design: redesign`.

Variants then change one thing each, switched on with `OMNICUI_DEFECTS`.
- **Look only (the behaviour is the same):**
  - `brand`: the same layouts in the reference design language (its colours and font);
  - `copy`: the value reads "3 taps";
  - `srlabel`: a visually-hidden "Count: " sits in front of the value;
  - `toolbar`: a heading and a Reset button that the contract doesn't name;
  - `pill`: a switch whose thumb slides up, over 400 ms, linearly.
- **Broken behaviour:**
  - `double`: each press adds two;
  - `divbutton`: a `<div role="button">` with no key handling;
  - `noevent`: no `toggled` event;
  - `nospace`: Space doesn't toggle.

The rules: the reference contracts are not changed (the contract experiments are copies, in `contract-variants/`), and no part is faked to get a pass.

```sh
./run.sh setup web
experiments/redesign/run.sh            # every run below (about 3 minutes); output in runs/<run>/
experiments/redesign/run.sh base       # one of them
```

## Results on 0.2.3 (webalt; `web` passes everything in every run)

| Run | webalt | Against the original's screenshots | Verdict |
|---|---|---|---|
| base (card + checkbox) | 9 pass, 1 fail | colours: 90–100% on 4 of 5, warns: `webalt uses #f4a261 (68%), #c77dff (22.2%), which web doesn't use` | right: a palette of its own is another design language. The fail is the thumb (the reference contract doesn't mark it optional; see a) |
| brand (the same layouts, the reference colours) | 9 pass, 1 fail | colours: **0%** on all 5 | right: another layout, the same design language |
| counter.copy ("3 taps") | 3 fail | | right by the exact contract; `matches` (e) lets it pass |
| counter.srlabel (hidden "Count: ") | pass | | right: `text` is what is shown (it failed on 0.2.2) |
| counter.toolbar | pass | | right |
| switch.pill (thumb slides up, 400 ms linear) | pass | timing 99 ms, clip 134 ms, **direction 90°: `web went right 20 px, webalt up 20 px`** | right (on 0.2.2, the direction wasn't noticed) |
| counter.double | 2 fail | | right |
| counter.divbutton | 3 fail | | right |
| switch.noevent | 2 fail | | right |
| switch.nospace | 3 fail | | right |
| fail-on-palette (contract a, the redesign) | exit 1 | 4 colour warnings | right: a CI gate on the design language |
| fail-on-palette-brand (contract a, brand) | exit 0 | 0 colour warnings | right: the same gate passes another layout in the same design language |

## Contract experiments on 0.2.3 (`contract-variants/`)

| Contract | webalt | What it shows |
|---|---|---|
| a: `thumb: { optional: true }` | **pass**, `no `thumb` in webalt's design (an optional part): its trace is skipped` | the fix: the motion check stays on the designs that have a thumb (failed on 0.2.2) |
| b: the trace and the clip in a test of their own, thumb not optional | that test fails, with ``if webalt's design has no `thumb`, mark the part `optional: true` `` | the message now names the way out |
| c: no trace, no clip | pass | still works, but loses the motion check; a is better |
| d: `record: { part: switch }`, no trace | pass; motion warning 78 ms (the pill: 134 ms) | a clip needs no part inside the switch |
| e: `text: { matches: "^3\\b" }`, with "3 taps" | **pass** | a design may show more than the contract cares about |

## On 0.2.2 (before the fixes)

| Run | webalt | What omnicui said |
|---|---|---|
| base | 9 pass, 1 fail; 5 pixel warnings, area 92–100% | the thumb's trace failed, and the hint (look the part up at each frame) was wrong for a checkbox |
| counter.copy, counter.srlabel | 3 fail each | `expected "5", got "5 taps"`; `expected "5", got "Count:\n5"` |
| switch.pill | pass, timing warnings only | the upward slide wasn't noticed |
| switch.noevent | 2 fail | the real failure was mixed with the thumb's false one in one test |
| `--fail-on pixels` | exit 1 | nothing could say webalt was meant to look different |
| contracts a, b | fail | `optional` covered masks and crops only |

## Still open

- The design language is compared by colour only: font, spacing, radii and elevation are not (GAPS 23).
- Text colours drawn only as glyphs rarely enter a palette (GAPS 24).
- A design's palette is what it drew in the run: a check of one component knows only its colours (GAPS 25).
