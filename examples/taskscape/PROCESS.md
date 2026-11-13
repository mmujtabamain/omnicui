# How the Taskscape example was made

Taskscape is a real app: a macOS task manager built as two Tauri apps. It was ported to GPUI and
to the web by agents, given only omnicui, its documentation, the app, and a written prompt each
(`prompts/`). This file records how it went, so the next example, and the next agent, can do it
better. The builders' transcripts and a summary of each are in `process/`.

## Who did what

| When (UTC) | Who | What |
|---|---|---|
| 12 Nov | the coordinator | Vendored the app (`tauri/`, its commit 09f3f4c), wrote WHY.md and the three prompts. |
| 12 Nov 19:15–20:55 | P0 (the coordinator, in its own session) | The inventory, eleven contracts, `platforms.yaml`, fixtures loaded through the app's Store, SPEC-PRODUCT.md, DESIGN.md, and the `tauri` target. Two full checks; committed. |
| 13 Nov 06:19 | P1 and P2, side by side | Each an agent of its own, from its prompt and the environment's working instructions. |
| 06:40 | P1 | Feasibility: a real GPUI window on a machine with no GPU and no display (Xvfb, Mesa's lavapipe), real input, real screenshots. |
| 07:18 | P2 | The web passes every contract but five, all P0's (tests expecting ⌘ on the web, and one contradicting the spec). P0 fixes them and commits P2. |
| 07:40–08:28 | P2, P0 | P0 adopts P2's proposals: a narrow web window, the browser's history, the capture bar in the page. P2's comparison finds the tauri target's capture screenshots blank; P0 fixes its target. P2's third check: 351 pass, 0 fail. |
| 08:12–09:27 | P1 | Three full checks with tauri: 353 pass, 0 fail. Committed after P0's review. |
| 09:30 | the coordinator | The transcripts, cleaned, in `process/`; the three-way check. |

## The result

The three-way check, every contract on all three targets (`omnicui check tauri gpui web`):

    527 pass · 0 fail · 0 broken · 38 not measured · 17 not applicable · 3 platform exceptions · 4 warnings

| target | its tests | time | start-up |
|---|---|---|---|
| tauri (the reference: the real app in Chromium) | 177 pass, 12 not measured, 5 not applicable | 10 min 39 s | 1.1 s |
| gpui (native) | 176 pass, 14 not measured, 4 not applicable | 4 min 04 s | 0.2 s |
| web | 174 pass, 12 not measured, 8 not applicable | 10 min 18 s | 1.0 s |

252 comparisons between the three (117 colours, 96 layout, 36 motion, 3 direction). The 4
warnings: the project menu's clip, gpui against both others (68 ms over the 40 ms threshold;
P1 measured 26–57 ms in its own runs, so it varies); and three between the two ports only
(gpui against web: the inspector's rise 24 ms, a draft's layout 10.4 %), each port within
threshold of the reference. "Not measured" are Settings' generated keyboard checks on parts
that appear only after a press; "not applicable" are each platform's own tests (the capture
bar's screenshot and window, the web's narrow window and history).

## What worked

- **A product spec and a design file written for the porters.** Both ports were built from
  SPEC-PRODUCT.md and DESIGN.md without asking a question about the product; their questions
  were about the contracts.
- **The reference target ran the real app.** The Tauri frontend in Chromium with its commands
  answered by the real Rust store meant the contracts were checked against the product as it
  is, not a reconstruction.
- **One fixture loader, through the app's own Store,** used by all three targets: the same data
  the same way, ids and all.
- **The ports caught P0's mistakes.** A test that contradicted the spec, four tests that assumed
  a Mac, an animation the app no longer has, and **screenshots that were blank but passed**, all
  surfaced only when a second target ran the same contract.
- **Limits stated up front** (three full checks, focused runs otherwise, gaps logged instead of
  worked around, never weaken a test). Neither builder faked a part to pass; both reported the
  contract as wrong instead, with what they would have written.
- **Each builder in its own folders, the contracts owned by one.** Disagreements came back as
  GAPS entries and were decided in one place.

## What didn't

- **A blank screenshot passed and was posted as a result.** The tauri target's capture bar never
  revealed (its reveal raced the listener), every capture test still passed because none read
  pixels, and its screenshot was posted to the team as the capture bar. The todo example's
  PROCESS.md warned about exactly this. Look at every screenshot you post.
- **A test that excluded the reference was never run** until a port ran it, and it was wrong.
  A test with `except: [<reference>]` needs a target that runs it before it is trusted.
- **Shared machine, shared folders.** The default run folder was shared, and one agent's run
  replaced another's; P0 rebuilt the tauri harness during P1's full check (10 false fails). Give
  every agent its own `--out` and don't rebuild a target while another agent's check uses it.
- **Disk.** P1's build folder reached 11 GB against a 10 GB limit before it trimmed it to 1.9 GB.
- **Times in PROGRESS.md written ahead of the clock**, by both builders: read the clock, don't
  estimate it.
- **An environment-only step.** The GPUI adapter needs the X11 libraries its machine keeps in a
  private folder; the first three-way check ran without it and gpui couldn't start.

## The gaps the agents found

See `omnicui/GAPS-P0.txt`, `gpui/GAPS-P1.txt` and `web/GAPS-P2.txt`. The ones for omnicui:
- prop defaults aren't sent with `mount`, so every registry repeats them;
- per-platform expectations exist for part fields, not events;
- no tab, treeitem or menuitem roles, no collection parts, no way to say a part is gone;
- motion that plays as a component mounts can't be traced;
- a title bar the app draws itself (with the window controls inside) doesn't fit the
  `titlebar` region, which assumes the platform's;
- `text` needs a rule for transformed and split text (upper case by CSS, a row of spans, icons
  from a ligature font);
- GPUI has no accessibility tree, so its parts' text is what the app declares.
