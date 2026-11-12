# Taskscape Main App Design Review

Date: 2026-08-13

## Scope

This review covers the `main` app shell, task panes, task rows, preview/filter/trash side panels, command palette, and settings window. It is grounded in source inspection and the provided screenshot, not live browser automation. Per project instructions, I did not run browser automation or claim behavioral verification.

- Deterministic Impeccable detector: clean on `main/src`
- Screenshot evidence used: yes
- Runtime/browser inspection: intentionally skipped
- Primary mode: `Operate`
- Assumed audience/tone for design judgment: individual power users, calm and crafted

## Method

Parallel Impeccable-style lenses were launched for critique, hardening, layout/copy, and delight, while this main pass inspected the codebase directly. This document is the consolidated review artifact you can use as an implementation backlog.

## Design Health Score

| # | Heuristic | Score | Key Issue |
|---|---|---:|---|
| 1 | Visibility of System Status | 3 | State exists, but active pane ownership, filtered emptiness, and load failure states are under-signaled. |
| 2 | Match Between System and Real World | 2 | The task-row “checkbox” defaults to selection, not completion, which fights user expectation. |
| 3 | User Control and Freedom | 3 | There is real control, but split-pane and right-rail role shifts make that control harder to read than it should be. |
| 4 | Consistency and Standards | 2 | Visual consistency is strong; interaction consistency with desktop norms is weaker. |
| 5 | Error Prevention | 3 | Task deletion is careful, but modifier-driven behavior and false-empty states still invite mistakes. |
| 6 | Recognition Rather Than Recall | 2 | Users must remember what icons, hover zones, and modifier-driven behaviors mean. |
| 7 | Flexibility and Efficiency of Use | 3 | Keyboard shortcuts, split view, palette, bulk actions, and drag/drop form a strong backbone, but some linked behaviors limit advanced workflows. |
| 8 | Aesthetic and Minimalist Design | 3 | Restraint is disciplined, but the interface often becomes visually under-expressive rather than elegantly minimal. |
| 9 | Help Users Recover From Errors | 3 | Task Trash helps, but note autosave and load failure recovery remain weakly exposed. |
| 10 | Help and Documentation | 1 | The product offers almost no in-product orientation, explanation, or reassurance. |
| **Total** |  | **25/40** | **Strong bones, but clarity and hierarchy are holding it back** |

## Design Specificity Verdict

Taskscape does not feel like generic SaaS. It is visually and mechanically more specific than that. The macOS-native chrome treatment, split-pane worldview, recessed project pill, inspector-first detail model, and dense power-user mechanics around selection, drag/drop, and hotkeys all feel authored for this product. The problem is not blandness at the system level; it is under-communication at the interaction level and under-expression at the emotional level.

The screenshot makes this especially visible. The left pane is doing almost all the work, while the right pane collapses into a large empty void with a small, low-consequence message. That creates a lopsided emotional read: the shell feels competent, but the product does not yet feel rewarding or especially memorable at rest.

## What’s Working

- The structural model is strong. The shell cleanly supports a primary pane, optional split view, and optional utility rail without collapsing into modal chaos. See [main/src/App.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/App.tsx:55).
- The app has a real power-user spine. Search scoping, bulk actions, drag/drop reordering, command palette, split panes, and settings search all signal serious intent. See [main/src/components/TaskPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskPane.tsx:204), [main/src/components/TaskRow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskRow.tsx:263), and [main/src/windows/SettingsWindow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/windows/SettingsWindow.tsx:155).
- The token discipline is unusually good. Surfaces, edges, content ramps, spacing, motion timing, and z-layers are systematized instead of improvised. See [common-ui/styles/tokens.css](/Users/obscure/Desktop/Git/taskscape-tauri/common-ui/styles/tokens.css:1).
- Settings information architecture is substantially better than the main window’s at communicating structure. Search-across-panes is especially strong. See [main/src/windows/SettingsWindow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/windows/SettingsWindow.tsx:155).
- The inspector is a strong product idea. Turning selection into notes, subtasks, attachments, and drafting context is one of the most distinct things about the app. See [main/src/components/preview/PreviewPanel.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/PreviewPanel.tsx:13) and [main/src/components/preview/TaskInspector.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/TaskInspector.tsx:258).

## Priority Issues

### [P0] The primary task-row mental model is upside down

- What: A plain row click previews, the round “checkbox” selects, `Option` changes it into a done control, double-click marks done, and keyboard behavior follows that same split. The cleverness is real, but the default control meaning fights the dominant task-manager mental model.
- Where: [main/src/components/TaskRow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskRow.tsx:328), [main/src/components/TaskRow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskRow.tsx:493), [main/src/hooks/useAppKeyboard.ts](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/hooks/useAppKeyboard.ts:40)
- Why it matters: Users will misread the core control, hesitate to trust bulk selection, and make avoidable errors around completion state.
- Fix: Make completion the default meaning of the checkbox, keep selection on row/meta/shift patterns, and frame bulk selection as a secondary expert mode rather than the default affordance.
- Suggested command: `$impeccable harden`, then `$impeccable clarify`

### [P1] Split view is capable but cognitively ambiguous

- What: Tabs represent both panes, pane ownership is only lightly marked, clicking an already-open tab can change focus rather than content, and filter edits can synchronize both panes unexpectedly.
- Where: [main/src/App.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/App.tsx:60), [main/src/components/ListTabs.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/ListTabs.tsx:100), [main/src/components/preview/FilterPanel.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/FilterPanel.tsx:87)
- Why it matters: This is one of the app’s signature features, so ambiguity here costs more than ambiguity anywhere else.
- Fix: Add a stronger focused-pane indicator, make left/right ownership more explicit in the content area, and keep filters independent by default unless the user explicitly links the panes.
- Suggested command: `$impeccable adapt`, then `$impeccable harden`

### [P1] The main canvas has too much dead air and too little intentional emphasis

- What: The primary split between task list and inspector creates a visually inert right side whenever nothing is selected, and even when populated the preview rail feels more like a utility drawer than a meaningful counterpart.
- Where: [main/src/App.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/App.tsx:108), [main/src/components/preview/PreviewPanel.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/PreviewPanel.tsx:42), [common-ui/src/components/content/EmptyState.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/common-ui/src/components/content/EmptyState.tsx:24)
- Why it matters: The app’s emotional center is currently absence. New or returning users see a large, dark panel that communicates “nothing happening” instead of “this is where your task gains depth.”
- Fix: Make the preview-empty state more editorial and useful. Show a stronger title, 2-3 possible next actions, and a subtle explanation of what the panel is for. Consider a richer default state with keyboard tips, selection hints, or a miniature task anatomy preview.
- Suggested command: `$impeccable onboard`, then `$impeccable layout`

### [P1] The app conflates empty, filtered, hidden, and failed states

- What: `TaskPane` can show “Nothing here yet” when tasks exist but are filtered out or hidden, and first-load failures can visually collapse into blank/stale surfaces because boot is dismissed in `finally` with no user-facing load error state.
- Where: [main/src/components/TaskPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskPane.tsx:78), [main/src/components/TaskPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskPane.tsx:308), [main/src/App.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/App.tsx:45)
- Why it matters: Users can mistake hidden data for absent data, or think the product is empty when it actually failed.
- Fix: Separate “no tasks,” “no results,” “filtered out,” “completed hidden,” and “load failed,” each with a direct CTA like `Clear filters`, `Show completed`, or `Retry`.
- Suggested command: `$impeccable harden`, then `$impeccable onboard`

### [P1] Core navigation is not semantically strong enough for keyboard and assistive use

- What: Task rows, list tabs, and project menu rows rely heavily on clickable `div`s and global keyboard handling rather than native interactive semantics.
- Where: [main/src/components/TaskRow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskRow.tsx:265), [main/src/components/ListTabs.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/ListTabs.tsx:128), [main/src/components/ProjectSwitcher.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/ProjectSwitcher.tsx:53)
- Why it matters: The app has a strong power-user keyboard substrate, but the primary surface still under-serves tab-only users, assistive tech, and standard focus expectations.
- Fix: Convert primary navigational surfaces to real buttons/tabs/treeitems with explicit focus management and selected state.
- Suggested command: `$impeccable harden`

### [P1] Visual hierarchy is too flat for the amount of information the app carries

- What: Titles, notes, badges, metadata, tabs, and side-panel headers all live in a narrow tonal and typographic band. Both `--font-sans` and `--font-display` resolve to `system-ui`, so hierarchy falls back to tiny size shifts and muted tones instead of a real reading voice.
- Where: [common-ui/styles/tokens.css](/Users/obscure/Desktop/Git/taskscape-tauri/common-ui/styles/tokens.css:123), [main/src/components/TaskRow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskRow.tsx:384), [main/src/components/preview/TaskInspector.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/TaskInspector.tsx:272), [main/src/components/TitleBar.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TitleBar.tsx:64)
- Why it matters: When everything is quiet, the eye has to work too hard to understand what matters right now.
- Fix: Introduce a real display/body distinction or a much stronger heading system, set a higher floor for persistent UI copy, and widen hierarchy bands in rows, inspector headers, and settings sections.
- Suggested command: `$impeccable typeset`, then `$impeccable layout`

### [P1] The shell communicates control more clearly than meaning

- What: The top bar is mechanically capable, but its icon-only tool cluster, tab density, and subtle active states prioritize control exposure over product framing.
- Where: [main/src/components/TitleBar.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TitleBar.tsx:40), [main/src/components/ProjectSwitcher.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/ProjectSwitcher.tsx:28), [main/src/components/ListTabs.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/ListTabs.tsx:95)
- Why it matters: The first horizontal band of the app should establish confidence instantly. Right now it says “frameless utility window” more strongly than “carefully authored task environment.”
- Fix: Increase distinction between project context, list navigation, and utility controls. Consider slightly more breathing room, stronger active tab treatment, less persistent delete emphasis in the tab strip, and a more unmistakable preview/split state treatment than filled icons alone.
- Suggested command: `$impeccable layout`, then `$impeccable delight`

### [P1] Settings are usable but emotionally and visually underpowered

- What: The settings window is organized well, but the experience is almost entirely composed of interchangeable rows and low-contrast separators. It feels more “internal tool panel” than “premium product preferences.”
- Where: [main/src/windows/SettingsWindow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/windows/SettingsWindow.tsx:201), [main/src/windows/settings/generalPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/windows/settings/generalPane.tsx:24), [main/src/windows/settings/aboutPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/windows/settings/aboutPane.tsx:5)
- Why it matters: Settings are where product values become explicit. If they feel visually generic, the product identity weakens.
- Fix: Keep the current IA, but give panes stronger personality through better section pacing, better intro copy, stronger selected-nav treatment, and richer explanatory footnotes where decisions matter.
- Suggested command: `$impeccable clarify`, then `$impeccable polish`

### [P1] The app’s response to failure is too quiet

- What: Bootstrapping, task loading, trash loading, and note autosave all have silent or weak failure paths, so the UI can look empty, stale, or complete when it is actually in error.
- Where: [main/src/App.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/App.tsx:45), [main/src/stores/bootstrap.ts](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/stores/bootstrap.ts:47), [main/src/components/preview/TaskInspector.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/TaskInspector.tsx:118)
- Why it matters: Calmness without truth becomes opacity.
- Fix: Add explicit bootstrap failure UI, inline retry states, and per-note save status or retry affordances.
- Suggested command: `$impeccable harden`

### [P2] Secondary panels are functional but not memorable

- What: Filter, Trash, and Command Palette all work, but each leans on the same neutral panel grammar without enough unique emphasis.
- Where: [main/src/components/preview/FilterPanel.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/FilterPanel.tsx:124), [main/src/components/TrashPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TrashPane.tsx:48), [main/src/components/CommandPalette.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/CommandPalette.tsx:64)
- Why it matters: When every auxiliary surface has the same visual temperature, none of them feels especially intentional.
- Fix: Tune each panel’s header, density, and accent usage around its job. Trash can tolerate more caution red. Filter can use stronger active-state storytelling. Command Palette can feel more “launcher” and less “generic overlay list.”
- Suggested command: `$impeccable colorize`, then `$impeccable delight`

### [P2] Copy is competent but rarely helpful

- What: Most labels are correct, but few are particularly guiding, reassuring, or vivid.
- Where: [common-ui/src/components/content/EmptyState.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/common-ui/src/components/content/EmptyState.tsx:38), [main/src/components/TaskPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskPane.tsx:308), [main/src/components/preview/PreviewPanel.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/PreviewPanel.tsx:51), [main/src/windows/settings/shortcutsPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/windows/settings/shortcutsPane.tsx:27)
- Why it matters: In a minimalist UI, copy has to carry more warmth and orientation.
- Fix: Rewrite empty states, action labels, and helper text to sound more intentional and more confidence-building without becoming chatty.
- Suggested command: `$impeccable clarify`

### [P2] Motion and authored micro-feedback are present in the system, but underused in the product

- What: The token system and motion utilities exist, but most visible surfaces still rely on a minimal rise/sheet-in vocabulary.
- Where: [common-ui/styles/tokens.css](/Users/obscure/Desktop/Git/taskscape-tauri/common-ui/styles/tokens.css:149), [main/src/components/preview/TaskInspector.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/TaskInspector.tsx:258), [main/src/components/CommandPalette.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/CommandPalette.tsx:64)
- Why it matters: The app has enough structural sophistication to deserve richer feedback, but not enough visible feedback to feel premium.
- Fix: Use motion sparingly but strategically around selection change, panel activation, bulk mode entry, and key confirmation states. Keep it short and utility-led.
- Suggested command: `$impeccable animate`

## Surface-by-Surface Findings

### 1. Window Shell and Frame

- The project switcher pill is one of the only clearly “crafted” moments in the chrome, and it should become the model for more of the shell. See [main/src/components/ProjectSwitcher.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/ProjectSwitcher.tsx:28).
- The drag-indicator glyph in the title bar is technically neat but visually cryptic. It reads as leftover app chrome more than meaningful affordance. See [main/src/components/TitleBar.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TitleBar.tsx:51).
- Preview, split, swap, settings, and reduced-motion states all compress into a tight icon cluster with similar visual weight. That makes the shell feel denser than the product beneath it.
- Active states in the shell depend too heavily on filled icons and subtle tone changes. They need one more channel: shape, divider treatment, or spatial emphasis.
- The shell’s personality is currently carried more by mechanics than by visual voice. The subdued palette is appropriate; the bigger gap is authored identity in empty states, typography, and panel emphasis.

### 2. List Tabs and Project Navigation

- The list tab strip is highly capable, but it looks busier than it feels because separators, hover states, drag states, split indicators, inline rename, close buttons, and the add button all live in a shallow visual band. See [main/src/components/ListTabs.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/ListTabs.tsx:118).
- Deleting a list on middle-click is a sharp edge that does not visually read as dangerous. See [main/src/components/ListTabs.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/ListTabs.tsx:132).
- The delete “x” on every tab adds friction to the silhouette of the tab row. It makes every tab look slightly disposable all the time.
- Project menu edit/delete controls appearing only on hover are efficient, but they also imply that the row itself and the row’s actions are different interaction layers without telling the user that.
- The split-pane markers are too subtle for how much state they are carrying. This is one of the most important clarity problems in the app.

### 3. Task Pane

- The search well is well-built functionally, but visually overloaded: search, scope options, match count, clear, divider, new task, and optional split-close all compete in a single row. See [main/src/components/TaskPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskPane.tsx:204).
- “Search {list name}” is correct but underpowered as copy. It does not teach scope or encourage use.
- The empty state correctly references capture hotkeys, which is good, but the treatment is too passive for such an important activation point. See [main/src/components/TaskPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskPane.tsx:308).
- The footer swap between stats and bulk actions is structurally smart, but the bulk bar can feel like a sudden mode change because the row above it does little to prepare you for selection mode. See [main/src/components/TaskPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskPane.tsx:328).
- Filtered-out tasks currently collapse into what looks like true emptiness, which is a serious trust problem. Empty, filtered, and failed are not clearly distinguished enough.
- The footer rhythm is too compressed. Passive stats and active bulk actions both live in a cramped utility strip, which weakens scannability.

### 4. Task Rows

- Task rows are technically excellent and conceptually crowded at the same time. See [main/src/components/TaskRow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskRow.tsx:343).
- The Option-key dual-mode checkbox is inventive, but too clever for its own good in a mainstream task row. The same control means “select” and “mark done” depending on a hidden modifier state. See [main/src/components/TaskRow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskRow.tsx:493).
- Child-count badges and attachment badges are useful, but they form a right-edge metadata crust that makes rows feel back-loaded.
- Disclosure chevrons disappearing until hover when expanded is elegant in isolation, but risky in long real lists because it hides structure to preserve cleanliness. See [main/src/components/TaskRow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskRow.tsx:413).
- Selected rows are mostly just blue-washed; they do not feel “claimed” strongly enough for a selection-driven product.
- The row separator starting at the text column is a nice detail and should stay. See [main/src/components/TaskRow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskRow.tsx:354).
- The row surface is not semantically strong enough. Too much primary navigation lives on clickable `div`s instead of clear interactive primitives.

### 5. Preview and Task Inspector

- The inspector title area is one of the better-resolved sections of the app: task title, provenance, timestamps, and action rail are cleanly arranged. See [main/src/components/preview/TaskInspector.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/TaskInspector.tsx:258).
- The selected-task tick on the left is subtle and tasteful, but currently too subtle to compensate for the overall quietness of the panel. See [main/src/components/preview/TaskInspector.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/TaskInspector.tsx:261).
- The action rail on the right side of the header is clear for experts but slightly “toolbox-y” in presentation. Four stacked small icons can feel mechanically dense.
- The notes section is powerful, but the UI around note creation and note count could do more to signal that notes are a first-class enrichment layer, not just an appendix.
- “Write a note ...” is serviceable copy, but the syntax with spaced ellipsis feels slightly draft-like. See [main/src/components/preview/TaskInspector.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/TaskInspector.tsx:417).
- When no task is selected, the preview panel does not adequately sell its value. See [main/src/components/preview/PreviewPanel.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/PreviewPanel.tsx:51).
- The inspector header is also too crowded for its importance. Title, metadata, and four vertically stacked actions compete in one compact block.

### 6. Filter and Trash

- The Filter panel is sensible and well grouped, but it reads more like an internal admin drawer than a tuned productivity instrument. See [main/src/components/preview/FilterPanel.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/FilterPanel.tsx:162).
- “Filter & Sort” is a generic header for a fairly rich surface. It could frame the current task set more explicitly.
- Active-state dots and reset affordances are helpful, but the panel still leans too much on text labels and segmented controls of similar visual weight.
- Trash is safer than many apps because it exists at all, but hover-only restore/delete on each row reduces immediate recoverability. See [main/src/components/TrashPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TrashPane.tsx:107).
- The Trash row height is so compressed that the second line is visually useful but emotionally undersold.
- Filter terminology like “Show,” “Refine,” and “Any / With / Without” is concise but slightly internal. It asks the user to translate the model.

### 7. Settings Window

- The settings search behavior is genuinely strong. Search-all-panes, match badges, and scoped narrowing are thoughtful decisions. See [main/src/windows/SettingsWindow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/windows/SettingsWindow.tsx:155).
- The selected left-nav state is correct but not especially affirming; it does not feel like a confident “section you are in.”
- `640x520` is serviceable, but visually the window feels a little cramped for the amount of explanatory text and control variety present. See [main/src/windows/SettingsWindow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/windows/SettingsWindow.tsx:32).
- The pane intros are underused. Shortcuts gets one; more panes should have a sentence that frames what the section helps you do.
- The About pane is functional, but there is a missed chance to communicate product character or trust cues there. See [main/src/windows/settings/aboutPane.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/windows/settings/aboutPane.tsx:5).
- Settings copy is polished but over-compressed in spots. Labels like “Search” and “Saved to” are efficient but not especially explanatory.

## Copy and Language Notes

- “Nothing here yet” is too generic for the core empty state.
- “No task selected” is accurate but emotionally dead.
- “Trash is empty” is fine, but could reassure more explicitly that deleted tasks will appear here first.
- “Restore defaults” in About is accurate, but the surrounding framing could be calmer and more confidence-building.
- “Done” as the completed filter label is concise, but the rest of the product otherwise tends to say “completed,” which creates a faint voice inconsistency.
- “Filter & Sort,” “Show,” and “Refine” are functionally correct but conceptually thin for first-pass scanning.

## Typography and Color Notes

- The app has almost no true display/body contrast because `--font-sans` and `--font-display` both resolve to `system-ui`. See [common-ui/styles/tokens.css](/Users/obscure/Desktop/Git/taskscape-tauri/common-ui/styles/tokens.css:122).
- Microtype is doing too much work. Repeated `10.5px` to `12px` labels make the app feel grayer and smaller than an operational task tool should. See [common-ui/src/components/typography/SectionHeader.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/common-ui/src/components/typography/SectionHeader.tsx:41) and [main/src/windows/SettingsWindow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/windows/SettingsWindow.tsx:321).
- Accent blue is semantically overloaded. It currently handles navigation, selection, drag targets, filter activity, split markers, inspector tick marks, and done-state lamps, which dilutes meaning. See [common-ui/styles/tokens.css](/Users/obscure/Desktop/Git/taskscape-tauri/common-ui/styles/tokens.css:51) and [main/src/components/TaskRow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TaskRow.tsx:530).
- The neutral palette is tasteful but too quiet in many resting states, especially for inactive-but-actionable controls.

## Accessibility and Desktop Ergonomics Notes

- Icon-only shell controls rely heavily on `title`, which is useful but not enough for glanceable comprehension. See [main/src/components/TitleBar.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TitleBar.tsx:78).
- Hover-revealed affordances risk discoverability loss for motor-impaired or low-precision users, especially in dense rows and menus.
- The global `user-select: none` posture gives a desktop-app feel, but it also makes the interface feel less forgiving. See [main/src/index.css](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/index.css:23).
- The app is clearly keyboard-aware in several places, but those capabilities are not surfaced enough to first-time users.
- Custom menus remain mouse-led in too many places, which weakens keyboard and focus behavior in filters, search scope, and context actions.
- The `800x600` baseline is fragile once split view and the side panel open together. At that size the app needs stricter minimum working widths or automatic secondary-pane collapse.

## Persona Red Flags

### Alex, Power User

- Loves split view, bulk selection, and command palette, but will likely stumble once on the Option-key checkbox duality before trusting it.
- May appreciate middle-click shortcuts in theory, but list deletion on middle-click is a trust breaker because it is destructive and visually unadvertised.
- Will want stronger at-a-glance differentiation between active pane, selected task, filtered state, and preview state.

### Jordan, First-Timer

- Will understand the left pane quickly, but may not understand why the right pane exists until selecting something.
- Will not infer hidden hover affordances reliably.
- May confuse “selected for preview” with “selected for bulk action” because the app uses quiet blue wash and subtle state transitions rather than stronger explicit mode framing.
- May also misunderstand the checkbox immediately, because its default meaning does not match common task-manager expectation.

### Morgan, Busy Professional

- Will appreciate the low noise at first glance, but may find the interface emotionally cold and operationally demanding over long sessions.
- Will benefit from stronger status narration: what is filtered, what is selected, what changed, what will happen next.

## Minor Observations

- The project switcher label cap at `max-w-44` can make longer project names feel clipped prematurely. See [main/src/components/ProjectSwitcher.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/ProjectSwitcher.tsx:39).
- The preview panel close icon uses `last_page`, which feels slightly opaque as a close affordance. See [main/src/components/preview/TaskInspector.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/preview/TaskInspector.tsx:347).
- The reduced-motion status icon in the title bar is thoughtful, but visually it reads more like a stray status glyph than a coherent part of the shell. See [main/src/components/TitleBar.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/TitleBar.tsx:65).
- The command palette is solid, but the bolt icon does not add much meaning relative to the text prompt. See [main/src/components/CommandPalette.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/components/CommandPalette.tsx:70).
- Attachment and subtask badges are numerically useful, but they could use clearer difference in tone or geometry so the eye reads them as distinct categories faster.
- Settings search results dim unmatched panes to `opacity-40`, which is functional but visually a little muddy. See [main/src/windows/SettingsWindow.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/main/src/windows/SettingsWindow.tsx:291).
- The shared `EmptyState` component is tasteful, but too generic to carry major product moments without custom children. See [common-ui/src/components/content/EmptyState.tsx](/Users/obscure/Desktop/Git/taskscape-tauri/common-ui/src/components/content/EmptyState.tsx:15).

## Questions to Consider

- Should Taskscape feel like a quiet instrument panel, or a more opinionated daily workspace?
  - A healthy mix of both
  - In future we can allow customization to the users
- Does the preview panel want to be a “details drawer,” or does it want to be a second half of the product?
  - Details Drawer
- Which interaction deserves to be the unmistakable hero: quick capture, row triage, or deep task inspection?
  - Quick Capture
- Is selection primarily a navigation concept, or a bulk-action concept? Right now it is doing both.
  - You decide
- What is the one visual motif, beyond blue accent and dark neutrals, that would make this unmistakably Taskscape?
  - None right now, you can create one
- How much cleverness is worth teaching before it starts costing trust?
  - You decide a healthy balance

## Direction From Your Answers

These answers sharpen the review into a working brief for the next design pass.

- **Overall product feel:** Taskscape should be a healthy mix of quiet instrument panel and opinionated daily workspace.
- **Preview panel role:** The right panel should behave like a details drawer, not the second half of the product.
- **Hero interaction:** Quick capture should be the unmistakable hero.
- **Customization stance:** The default experience should have a strong point of view now, with user customization as a later layer.
- **Selection decision:** Selection should primarily be a navigation concept. Bulk action should exist, but as a clearly-entered secondary mode.
- **Cleverness decision:** Cleverness is acceptable only when it improves speed without changing the default meaning of primary controls or hiding destructive consequences.

## Design Direction

### 1. Quiet Shell, Opinionated Core

The shell should stay calm, dense, and desktop-native. The opinion should come from:

- stronger quick-capture emphasis
- clearer task-state storytelling
- a more authored typographic hierarchy
- one repeatable visual motif

The app should not become louder overall. It should become more decisive.

### 2. Quick Capture Becomes the Hero

If quick capture is the hero, the main app should feel like the place where captured material gets clarified, triaged, and deepened.

That means:

- empty states should point back to capture more often
- the shell should give capture more narrative importance than filter, trash, or settings
- freshly captured tasks should feel visibly “received” by the system
- list view should read as a triage surface first, with inspection secondary

### 3. Preview Rail Stays a Details Drawer

Because the right side is a details drawer, it should stop behaving like a rotating identity slot for preview, filters, trash, and draft states.

Implications:

- preview remains the default and most stable meaning of the rail
- filter and trash should move closer to utility surfaces, overlays, or clearer secondary destinations
- the drawer open/close model should become more obvious
- the empty preview state should be useful but non-dominant

### 4. Selection Means “I’m Looking At This”

My call is that selection should primarily mean navigation/focus, not bulk-action staging.

That implies:

- the main circular row affordance should default to completion, not selection
- row click should focus and inspect
- multi-select should be explicit through standard desktop patterns like `Shift` and `Cmd`
- bulk actions should feel like a mode entered on purpose, not the default hidden interpretation of the row control

This fits your quick-capture priority better: captured tasks become easy to process immediately without teaching a custom selection grammar first.

### 5. Healthy Cleverness

My recommendation:

- keep cleverness in shortcuts, palette depth, drag/drop, and optional expert flows
- remove cleverness from primary control meaning, destructive actions, and state interpretation

Practical rule:

- a user may discover a faster way later
- a user should not have to discover the correct way later

That means the following should be obvious on first use:

- what checking a task does
- what selecting a row does
- what the right drawer is for
- what is filtered vs truly empty
- what can be undone vs what is permanent

## Proposed Visual Motif

Use the **rounded checkbox / check mark** from the loading screen as the base brand form, then refine it into a full product motif.

Base reference:

- the current loading mark in [main/index.html](/Users/obscure/Desktop/Git/taskscape-tauri/main/index.html:47)
- the matching native boot box in [main/src-tauri/src/panels.rs](/Users/obscure/Desktop/Git/taskscape-tauri/main/src-tauri/src/panels.rs:113)

Concept:

- a rounded square frame with a confident check stroke inside
- calm, precise, and slightly luminous rather than playful
- capable of feeling like both a captured item and a resolved item
- recognizable enough to become the product’s repeated visual anchor

How to evolve it:

- keep the overall silhouette of the loading checkbox
- refine the corner radius, stroke rhythm, and proportion so it feels more authored
- let subtle motion or line-trace behavior animate the check when capture or completion happens
- use any “signal” or “trace” language as a behavior layered onto this mark, not as a separate symbol

Where it can appear:

- quick-capture entry and fresh-capture states
- app boot and loading transitions
- active focus or selected-task accents
- empty states about capture, triage, and task readiness
- subtle shell status moments or onboarding cues

Why it fits:

- it already exists in the product, so the motif grows from real UI rather than an invented metaphor
- it reinforces quick capture and task resolution at the same time
- it stays quiet enough for a desktop productivity app
- it gives Taskscape a repeatable authored signature beyond “dark UI with blue accent”

## Updated Priority

Given your answers, the next design work should be reprioritized like this:

1. Reframe the task row model so completion is primary and selection is clearly secondary.
2. Rework the right rail into a stable details drawer and move competing utilities out of its identity.
3. Elevate quick capture in the shell, empty states, and post-capture task handling.
4. Introduce the checkbox-derived motif plus stronger typographic hierarchy.
5. Reduce hidden cleverness in primary flows while preserving expert speed in shortcuts and secondary interactions.

## Recommended Next Artifact

The highest-value next artifact is a concrete redesign brief for:

1. Title bar and shell hierarchy
2. Task row interaction model
3. Preview drawer behavior
4. Empty states and quick-capture emphasis
5. Signature visual motif and typographic hierarchy

## Recommended Command Sequence

1. `$impeccable layout` for shell hierarchy, pane balance, tab emphasis, and preview-empty composition
2. `$impeccable harden` for hover/modifier/destructive affordance cleanup
3. `$impeccable clarify` for empty states, task-panel copy, and settings language
4. `$impeccable typeset` for stronger hierarchy in rows, panels, and settings
5. `$impeccable delight` for authored identity moments that do not hurt productivity
6. `$impeccable animate` for restrained but meaningful feedback
7. `$impeccable polish` as the final consolidation pass

## Limits

- This review does not confirm runtime behavior, pointer feel, animation timing, screen reader output, or real-world contrast on-device.
- Browser automation was intentionally not used because the repo instructions reserve behavioral and visual verification for the user unless explicitly requested.
