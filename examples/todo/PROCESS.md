# How the todo example was made

The todo example was built by a separate agent, given only omnicui, its documentation and a written brief. Two more agents then added the iOS and macOS versions from the specs it left. This file records how it went, so the next example (and the next agent) can do it better. The agents' full transcripts are kept with the project's records, outside this repository.

## Who did what

| When (12 Nov 2026, UTC) | Who | What |
|---|---|---|
| 09:23 | the sub-agent | Started from a brief:<br>- the documents to read first;<br>- an Android app (Compose, Material 3) and a web app, each with its omnicui target;<br>- shared contracts, and SPEC-APP.md for the platforms it would not build;<br>- rules: one folder, at most three full checks, no faked parts, gaps logged, no commits. |
| 09:41 | the sub-agent | Android app builds. |
| 09:56 | the sub-agent | The Android target works by hand: one JVM running Robolectric, so Gradle never starts during a check. |
| ~10:00 | — | The sub-agent stopped: the account's usage limit. It resumed at 10:12 with its context intact, when asked. |
| 10:15 | the sub-agent | The contracts are written on 0.2.4's per-platform features, which landed in omnicui while it worked. |
| 10:31 | the sub-agent | The web app and target pass every contract. |
| 10:40 | the sub-agent | Final check: 155 pass, 0 fail, 1 warning (the switch's motion, a real design difference). |
| 10:50–11:08 | the sub-agent | Redone three times, as the person asking clarified the design split (below). |
| afternoon | two more agents | iOS (SwiftUI) and macOS (SwiftUI and AppKit), each from its SPEC-APP.md, on a Mac. |

## How the design split was settled

The intended split took four messages to pin down. Each one changed the contracts, not omnicui:

1. "Web and macOS pixel-identical, mobile entirely native": the first reading.
2. "No mobile ↔ desktop checks": the mobile and desktop apps became two separate omnicui projects.
3. "Not entirely pixel by pixel; web like the web, desktop like the desktop": the desktop pair compares layout, colours and motion, never pixels.
4. "Use Joy UI", then "skip the Joy UI instruction": built, then removed.

omnicui gained two things from this, both general:
- a contract-level `same:` per platform, honoured across designs;
- the cut-off of full-width title and status bars, so a macOS window's content lines up with the web's.

**Lesson:** settle the split as a table before any building. For each pair of platforms, decide pixels, layout and colours, behaviour only, or no check at all.

## What worked

- **A brief that names what to read**, and the earlier review of an agent's mistakes. The sub-agent avoided every one listed there:
  - a Gradle start per check;
  - whole screens as one component;
  - XCTest for every read.
- **Limits stated up front:** at most three full checks, focused runs otherwise, gaps logged rather than worked around.
- **Specs for the platforms it could not build** (SPEC-APP.md). Both later agents built from them alone.
- **Screenshots at every milestone**, posted to the team as they happened. The person asking could follow and correct the design while it was being built.
- **Checking the agent's results independently.** Every summary line was re-run before it was committed.

## What didn't

- **Screenshots attached from a local path never reached the team.** They must be uploaded first and linked.
- **No deliberate defects.** Nothing showed that the contracts fail when the behaviour breaks.
- **The iOS evidence screenshots came out blank**: four identical black images. The mobile contracts compare no visuals, so nothing caught it.
- **A usage limit stopped the sub-agent mid-run.** A brief that keeps PROGRESS.md current makes that cheap to resume.

## The gaps the agents found

See GAPS-todo.txt, mobile/GAPS-ios.txt and desktop/GAPS-macos.txt. The ones that recur:
- generated keyboard tests can't open a dialog or choose another segment first;
- a test can't say a part is absent;
- there are no collection parts (rows are addressed by position);
- there is no appearance check;
- a blank screenshot passes unnoticed when no visual comparison applies.
