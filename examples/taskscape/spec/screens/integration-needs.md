# What the screens need from the main window and shared code

Collected from the wave 1 and 2 screen builders' reports ("needs").

## trash

Builder: 1) The Empty Trash sheet is not drawn by the app's sheet host: SheetSpec in screens/sheets/store.ts has no emptyTrash kind, so TrashPane renders its own EmptySheet card (same widgets and look as SheetHost, parts app.trash.emptySheet*). To make it the app's sheet host, add a { kind: "emptyTrash"; count: number } spec there (glyph delete_forever, danger, detail "<n> tasks deleted for good. No undo.", accept "Empty Trash", Cancel first). Owner of screens/sheets to decide. 2) Button has no danger variant that is red at rest as text; I used variant ghost with tone danger instead of variant danger (filled red). If the team wants the filled look, the Empty button changes to variant danger. 3) Contract keyboard tests for cancelEmpty and confirmEmpty are not measured because the sheet is not on screen when app.trash mounts; a harness prop that opens the sheet would let them run. 4) Harness events are listed as strings ["restored","purged","closed"] following the AGENTS example shape; the runner accepted it.

Fixer: 1) screens/sheets/store.ts (owner: sheets) has no emptyTrash SheetSpec, so TrashPane draws its own EmptySheet card. To move it to the app's SheetHost, add { kind: "emptyTrash"; count: number } with glyph delete_forever, danger tone, detail "<n> tasks deleted for good. No undo.", accept "Empty Trash", Cancel first. 2) Contract keyboard tests for cancelEmpty and confirmEmpty cannot run because the sheet is not mounted when app.trash mounts. The runner suggests making the sheet its own component with keyboard: true; that is a contract or harness-structure decision, not a change this screen can make. 3) Contract limit: one element carries one data-part. When the printer row is first, restorePrinter and purgePrinter sit on wrapper spans (role generic, forwarding Enter and Space to the button), because restoreItem and purgeItem sit on the button. The contract owner should decide whether the positional parts stay or the printer row's named parts are reached another way.

## palette

Builder: 1. Store or shell support is missing for these commands. They close the palette and report their label, but do nothing else: Collapse all and Expand all (no collapsed-set in useUi); Search this list (no focus-search signal); Undo and Redo (no shared undo history); New list and New project (no sheet state in the store). Each needs a store field or action from its owner.
2. Delete “<task>” calls tasks.trash directly. Decision 10 requires a danger confirmation (Cancel focused). That needs a confirm path (sheet state in the store, or a shared confirm action).
3. The app shell must open the palette: set useUi.paletteOpen from Ctrl/Cmd+K and the title-bar control. That is outside this screen.
4. Widget looks that cannot match the reference without restyling: InputGroup's default field fill on the palette input row (reference is white; a tone or bare variant for the bar shape would fix it); ComboboxItem icon tone is inherit (reference grey); ComboboxItem hotkey platform comes from isMac, so Linux renders text labels.
5. Role mismatches: the contract says the row role is button, but ComboboxItem is role option. I set role="button" on each row in the screen to satisfy the contract; the spec owner should confirm. The search input's role is combobox (InputGroup's bar shape) while the contract says textbox; the test passes, but the widget's role is not textbox.
6. Harness note: the runner resolves a part by data-part, and it also matched an element with data-slot="row" that came earlier in the DOM. The input row's data-slot was renamed to palette-search to fix this. Worth a line in the widget or runner docs.

Fixer: 1. Store or shell support for the no-op commands: collapsed-set for Collapse all and Expand all; a focus-search signal for Search this list; a shared undo history for Undo and Redo; sheet state for New list and New project. Each needs a store field or action from its owner.
2. Delete "<task>" calls tasks.trash directly. Decision 10 requires a danger confirmation with Cancel focused. This needs a confirm path (sheet state in the store or a shared confirm action) before it ships.
3. New task should start an unsaved draft in the inspector, as the reference does. This needs a draft action in the store; until it exists the palette creates a real "Untitled" task.
4. The app shell must open the palette: set useUi.paletteOpen from Ctrl/Cmd+K and the title-bar control. This is outside this screen.
5. Role mismatches for the spec owner to confirm: rows are role="button" (set on each row to satisfy the contract, though ComboboxItem is role option), and the search input is role combobox where the contract says textbox.
6. Widget looks that cannot match without restyling: InputGroup's default field fill and divider colour on the palette row (a tone or bare variant for the bar shape would fix it); ComboboxItem icon tone is inherit (reference grey); ComboboxItem hotkey platform comes from isMac, so Linux shows text labels.
7. Runner note: the runner matched data-slot="row" before the data-part element; the input row's data-slot was renamed to palette-search. Worth a line in the widget or runner docs.

## multiinspector

Builder: AppHarness (apps/web/frontend/src/harness/lib/AppHarness.tsx, shared and read-only for me) does not mount TooltipProvider, but Button and Checkbox tooltips need one. My harness wraps the screen in TooltipProvider as a workaround. Suggest adding TooltipProvider to AppHarness so every screen's harness gets it. Nothing else is needed.

Fixer: 1. AppHarness (apps/web/frontend/src/harness/lib/AppHarness.tsx, read-only for me) does not mount TooltipProvider, but Button and the new Checkbox tooltip need one. The harness wraps the screen in TooltipProvider locally. Adding TooltipProvider to AppHarness would let every screen drop that wrapper.
2. Review decision: the harness sets the last selected task as the active row to match the reference highlight. The contract notes say the harness should not set an active row. This leaves Enter and Space on lastRowTitle not measured. If the team keeps the notes' rule, the highlight in the screenshot tests will not match the reference.
3. SectionHeader widget: its label is 11px with 0.08em tracking, not 10.5px with 0.12em. Matching the reference needs a size or tracking option on the widget. Owner: packages/ui widgets.

## appbar

Builder: 1) The contract's keyboard: true cannot measure Enter/Space on project rows, tab menu items, swap panes and account items because they are not on screen at mount. Either the contract lists them as optional, or a shown-state harness prop is added (contract is read-only for me). 2) screens/palette and screens/trash do not exist, so the bar opens the palette and trash through the useUi store only (setPalette, setAside("trash")); the owners of those screens must read that state. 3) Export list only emits listExportRequested; the download belongs to the list export owner. Sign out only emits signOutRequested; the app shell must do the sign-out. 4) A failed inline rename is swallowed silently; a shared error toast would be needed to meet the never-silent rule.

Fixer: 1) Keyboard on menu rows (projectRow, projectRowSpring, rename/delete rows, tab menu items, swap panes, account items) is still not measured: the rows are not on screen at mount. The contract is read-only; it needs those rows marked optional or a shown-state prop, or a sheet/dialog component that mounts them shown.
2) Palette and Trash: nothing is needed from their owners. The bar sets useUi paletteOpen through setPalette(true), and settings through setSettings(true). The palette screen reads paletteOpen. The trash pane reads aside, which the bar sets to "trash" with setAside.
3) Export list only emits listExportRequested; the JSON download belongs to the list export owner (lists.export exists in endpoints). Sign out only emits signOutRequested; the app shell must call auth.signout.
4) A failed inline rename is swallowed silently (commitRename). web.md requires an inline or toast error; the toast store has no plain error notice (failToast needs an existing undo notice id), so a shared error toast API is needed.
5) Dropping a task on a tab reads the task id from the MIME type application/x-taskscape-task (comma-separated ids). The task list owner must set that payload when it starts a task drag, or the drop does nothing.
6) The contract notes list drag-and-drop as left out, but I implemented tab reorder and task drop because the features list F02-053 to F02-056 says the bar does them. The contract does not measure them.

## tasklist

Builder: 1. Harness: the modifierHold and selectToggle input steps do not set Alt/Control state on the page's key or pointer events, so the Option-held and Ctrl-pick tests cannot pass from the screen. 2. Sheets: there is no multi-task deleteConfirm spec, so bulk Delete is disabled unless exactly one task is picked. 3. Harness: submenu hover on menuMoveToList does not open its submenu under the runner; check whether the runner's hover reaches pointerenter.

Fixer: 1. Runner (check-screen step semantics): modifierHold (Alt) and selectToggle (Ctrl or Cmd) must set altKey/ctrlKey/metaKey on the page's key and pointer events and send keydown/keyup for Alt/Control/Meta to the page. Four tests depend on it: option held and hovering a row, Option-click on the check, Option-click on a parent, Ctrl press on a row body.
2. Runner: hover on a part must send pointerenter/pointermove/mouseenter/mousemove to the element. Needed for the Move to list submenu test (menuMoveToList).
3. Runner or contract: the two scroll steps on scrollArea must bring Errand number 40 into view (48 rows, about 1,500px at 32px rows). Either the scroll step is larger, or the contract needs more steps.
4. Sheets: no multi-task deleteConfirm spec, so bulk Delete is disabled unless exactly one task is picked (feature F01-030 is only partly met).
5. Not built: drag and drop (features F01-020, F01-021, F07-037 and related). The contract leaves it out.

## taskinspector

Builder: 1. RichTextEditor (packages/ui rich-text-editor.tsx): (a) forward a data-part prop to the textbox, toolbar Toggle/Button elements, mention list, option rows and mention chips, or expose them; the screen currently tags them from the DOM. (b) insertMention should add a trailing space after the chip (F03-043). (c) MentionView must render inside NodeViewWrapper from @tiptap/react, which currently logs an error on every mention render. (d) The character counter is only shown above 90% of maxLength; the screen draws its own counter, but a count prop or always-on counter would be cleaner.
2. Note storage: the API stores note content as HTML, but the editor reads and writes plain text, so bold, lists and links are lost on the first save. Needs either an HTML round trip from the editor widget or an agreed plain-text storage rule.
3. Contract: the focus-caret test ("saves its edit when it loses focus") conflicts with the mention tests about where the caret sits on focus; the plain-Enter test conflicts with F03-028 (note created on first keystroke). Owner to decide which wins.
4. Runner timing: "no part" misses on direct data-part buttons in some runs; a readiness signal from the harness would help, or the runner should retry parts longer.
5. Contract mentionItem is listed as role button, but the widget renders role option (the contract comment says option).

Fixer: 1. RichTextEditor (packages/ui rich-text-editor.tsx): (a) insertMention should add a trailing space after the chip (F03-043); (b) MentionView must render inside NodeViewWrapper from @tiptap/react; this logs an error on every mention and probably causes the paragraph split in the trailing-space test; (c) the formatting bar needs dividers (F03-053), the Link button should be named "Add link" (contract noteLink), and a character count inside the bar (F03-054), not under the editor; (d) optional: forward data-part to the textbox, toolbar, mention list, options and chips (the screen currently tags them from the DOM, which works). 2. Note storage: the API stores HTML and the editor reads and writes plain text, so bold, lists, links and mention chips are lost on reload. Needs an HTML round trip from the editor or an agreed plain-text rule. 3. Owner: the focus-caret test conflicts with the @-mention tests (see the summary); the plain-Enter test conflicts with F03-028. 4. Owner: F03-032 asks for debounced autosave, but the contract's autosave test reads the save straight after typing; pick one (a debounce needs the runner to wait for it). 5. Contract: mentionItem is listed as role button, but the widget renders role option (the contract comment says option).

## settingsshortcuts

Builder: 1) Keycap widget (packages/ui/src/components/ui/keycap.tsx): the button's accessible name is the raw accelerator, so a row's key cannot say "Left arrow" or "Command" (F08-075). The screen overrides the name on the button from its own code. The widget should take a spoken-name prop, and its data-part/parts should reach the button, not the wrapper div. 2) Settings shell: the General and About panes have no content here and General's rows are not searchable from this screen; the search-across-sections rule (F10-012, F10-015) needs the General rows supplied by the shared shell. 3) Command ids for the 31 Main window rows were taken from spec/features records; the API's bind() should confirm them (the tests exercise new_task, search, undo, redo, switch_list_1, new_list, prev_tab and prev_project; the others are unverified). 4) SectionHeader draws the changed dot whenever active=true; the screen uses its own group header to show only the undo control (review note 12), so the widget is unused here.

Fixer: 1) Keycap widget (packages/ui/src/components/ui/keycap.tsx): the button's accessible name is built from the raw accelerator, so the screen overrides aria-label from its own code. The widget should take a spoken-name prop. Its visible text is HotkeyHint text such as "Ctrl Alt ArrowDown", not spoken or glyph keys. Key capture uses e.key rather than e.code, which breaks Option/Alt letter combos on macOS (F08-069). The data-part is applied to the button through useParts, which works. 2) Settings shell: the General and About panes have no content here, so typing in search does not reach their rows (F10-012, F10-015, F10-016). The shared shell must supply those rows. 3) Command ids for the 31 Main window rows come from the feature records and are unverified; the API's bind() should confirm them. 4) Contract: groupMain text is checked as rendered text, so CSS uppercase (the reference look, F10-035) fails "Main window". The contract should accept text-transform, or the reference look should be accepted as small caps. 5) Contract: discardAll, searchClear, discardGroup and reset are not on screen at mount, so their keyboard checks are not measured. Pressing tabShortcuts has no readable effect either. The contract also warns that parts pane and row are never used by any test.

## settingsabout

Builder: 1. SettingsDialog.tsx (read-only for me) renders <AboutPage /> as its About pane. AboutPage now renders the whole dialog (header, search, tabs and About content), so the main web Settings page would show a second header and search. SettingsDialog needs to render an About-pane-only part, or AboutPage needs an export for that. 2. SettingsDialog passes no version. No version source is in the listed sources, so the version row shows an em dash there until the app supplies ctx.version. 3. Owner copy for the Application description is still needed; the placeholder is in AboutPage.tsx.

Fixer: 1. SettingsDialog.tsx (read-only for me) renders <AboutPage /> as its About pane. AboutPage now draws the whole dialog (header, search, tabs and About content), so the main Settings page shows a second header and search. SettingsDialog needs an About-pane-only export from AboutPage.tsx, which I can add if wanted. SettingsDialog also renders <ShortcutsPage /> inside its own body, which has the same nested-dialog problem.
2. ShortcutsPage.tsx renders a full Overlay dialog, so AboutPage cannot host it as the Shortcuts pane. It needs a pane-only export. Until then the Shortcuts pane body is empty in AboutPage.tsx.
3. SettingsDialog passes no version, so the version row shows an em dash there until the app supplies ctx.version.
4. Owner copy for the Application description is still needed. The placeholder is in AboutPage.tsx (APP_DESCRIPTION).</needs>


## signin (wave 1)

Fixer: 1) Data (contract or runner, not the screen): the create-account tests run in light and dark within one run and reuse the same addresses. Give each colour scheme a fresh database, or generate unique emails per run. The screen cannot pass a refused duplicate. This is the only reason the two tests fail.
2) Event hook: the harness at apps/web/frontend/src/harness/signin.tsx still takes the first present of props.emit, props.onSignedIn, window.omnicuiEmit, or the window CustomEvent "omnicui:signedIn". I could not read the runner, so I did not trim it. The sign-in success tests pass in both schemes, so at least one of these is reached. Name the real hook and I will reduce the chain to it.
3) Password masking: the Input widget (packages/ui/src/components/ui/input.tsx) hard-codes type="text" after its spread, so the password field is masked with a class. A type prop on Input would be the proper fix. This is a widget change, so it is needed from the widget owner.
4) Reference screenshots: none were named. Supply their paths if a pixel comparison is wanted.
5) Contract warnings only: the parts card, nameError and passwordError are not used by any test. The contract is read-only, so nothing changed here.

## sheets (wave 1)

Fixer: 1) Backend: DELETE /api/projects/{id} returns 500 ("'SimpleLazyObject' has no attribute 'objects'"). Fixing it clears the two failing delete-project contract tests.
2) Toast widget (packages/ui/src/components/ui/toast.tsx): pass data-part through to the root (ToastRoot) and to the Undo and close Buttons, so the Moved to Trash toast can use the widget instead of the div in SheetHost.tsx. The Toast widget's default duration is 5000 ms, while F06-095 says 6 seconds, so the sheet passes duration 6000 once the widget takes it.
3) Dialog and Toast widgets have no data-part pass-through. Overlay, Scrim, Button, Icon, Text, Input and DialogButton do pass it.
4) Footer labels: the sheet footer needs 12.5px semibold text in a 28px button (the reference look). DialogButton fixes the Button size to lg (14px medium), and no size token gives 12.5px. A sheet-footer size or variant in the widget is needed; I did not restyle it from outside.
5) MainWindow (screens/main-window/MainWindow.tsx) is a stub, so the scrim sits over a plain surface, not the main window. Once it is built, it should render from the useUi store that the harness sets for delete-confirm.
6) The runner's "answered" sink is the emit prop (called as emit("answered", payload)). That is now the only sink in harness/sheets.tsx. This is confirmed by the check, not from the brief.

## toasthost (wave 1)

Fixer: 1. Contract (read-only): the keyboard checks for undo and dismiss (Enter and Space) cannot run because app.toasthost mounts with no notice. Add a prop that mounts the notice shown, or keyboard checks that run with trash set. 2. Shared Button widget: the reference Undo is 12px semibold accent. Button has no size or variant for that at this box, so a variant or size for small semibold accent text would let it match. 3. Decision: should the notice pause on hover or focus? The reference does not, the widget Toast does, and keyboard users can lose Undo after 6 s. 4. Decision: the failed-Undo wording ("Could not undo the delete.") is a placeholder; decision 11 does not settle it. 5. Confirm the main window's visual order of Errands roots. The harness uses API sort_order, which gives Buy groceries first as the contract expects, but the reference screenshot lists Pick up prescription first.

## attachmentlightbox (wave 1)

Fixer: 1. Events: confirm the runtime's event convention (callback prop named after the event, its on-prop, or emit). The harness currently forwards to all three, and only one is live. Trim it to the confirmed one.
2. Contract "closed.via": stage clicks report "stage", which the contract does not list. Add "stage" to the documentation, or name the value a stage click should report.
3. Scrim: the Lightbox widget's Scrim renders mid-grey over the blank page, and the reference backdrop is the dimmed main window. A match needs the main window mounted behind the lightbox (outside this screen) or a stronger shared scrim.
4. Fallback card button: the review decision asks for the shared onMedia Open externally widget, but that variant's white text is unreadable on the light card. Decide whether the card gets a dark surface, or keeps the primary Button.
5. Lightbox widget: add an onOpenAutoFocus (or initialFocus) prop, so the first top-bar button does not take a focus ring on mount.
6. API: an endpoint that returns attachment text would replace the direct fetch() in the text preview branch.

## filterpanel (wave 1)

Fixer: Decision on the demo seed (read-only for me). The reference screenshot shows Errands in this order: Pick up prescription, Post the parcel to Mum, Book bike service, Cut a spare key, Buy groceries for the week (with its children). The contract's manual-order expectations (rest first row Pick up prescription; search "b" first row Book bike service; Manual + Descending first row Buy groceries) agree with it. The demo seed's stored sort_order (then id) puts Buy groceries for the week first, and web.md also lists it first. The screen keeps the stored order, as F05-044 says. To pass the five order tests, the seed's sort_order for the Errands root tasks needs to be Pick up prescription, Post the parcel, Book bike service, Cut a spare key, Buy groceries (Buy groceries last). Alternatively the contract expectations change. The seed is the one I recommend, because two sources (reference and contract) agree. Also needed: the regression fix in TaskPane.tsx (the title Button now carries block truncate) should be confirmed by one more check run, since no run remains for me.

## draftinspector (wave 1)

Fixer: 1) Widget changes needed (not made from outside):
 a) Input: the title keeps focus:ring-1 focus-ring-focus-1; the reference shows none. Needs a prop to drop the focus ring for the draft title.
 b) Input multiline: capped at max-h 3rem (two lines, then scroll). The reference grows to full scroll height, and the long-title screenshot shows a clipped title. Needs a max-height prop or a larger cap.
 c) Input mode "draft" fills bg-surface-0 (white). The reference title sits on a light grey wash. No widget mode gives that; mode "bare" never commits.
 d) Button dashed variant has a transparent fill (reference is white), and has no weight prop (reference label is 600, Button uses medium).
2) Keyboard trade-off (product decision): Discard, Note and Image are tabbable={false}, so Tab from the title leaves the draft, as the contract's "Tab away commits" tests require. Keyboard users cannot Tab to these three. The alternative is to change the contract's Tab stand-in and keyboard tests. Please confirm.
3) Image file chooser: the screen creates a hidden input with document.createElement("input"). This passes lint only because it is not JSX, so it sidesteps the lint rule's intent. Please review it, or provide a FilePicker widget.
4) Writes are unverified and swallowed: the Note empty-content write (notes.create with ""), and the Image upload, were not exercised against the real API. Failed writes are not shown, although web.md says failures must show inline or in the toast. The draft has already closed by then, so surfacing the error needs a shared toast or store hook.
5) animate-rise on the root makes the light sheet look faded in the captured frames. The capture is probably mid-animation. Not verified against the reference timing. Removing the class would drop the entrance animation (F04-014), so I kept it.

## settingsgeneral (wave 1)

Fixer: 1) Shared, read-only, for the owners: the four viewport/scheme combinations share one demo account, so concurrent runs can still change each other's stored settings; run this screen's tests serially, or give each test its own account. The reset gate in the harness entry is my workaround; the runner's own reset should also land before the page loads. 2) AppHarness.tsx creates a new QueryClient inside its render body; a re-render would drop the cache. Not seen in this run; flagged for the shared-code owner. 3) Switch widget (packages/ui switch.tsx): a narrower size variant is needed to match the reference's 30px toggle, since its thumb travel is fixed to the 2.5rem width. 4) Contract: appearanceGroup and tasksGroup are unused by any test (warnings only); the keyboard checks on parts hidden at mount cannot be measured as the contract stands.
