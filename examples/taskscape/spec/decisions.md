# Taskscape rebuild: decisions

Product decisions for the rebuild, with the reason. "Web" is the Django + React version, "desktop" the offline one. Earlier decisions (feature list review, 13 Nov) come first; the flowchart review answers (14 Nov) follow, numbered as in that review.

## From the feature list review

- Task due dates were never built; they are dropped.
- Selecting rows uses the platform modifier: Control-click on Windows and the web, Command-click on macOS.
- Option (Alt on Windows and the web) switches the round check from select to done.
- An untitled draft is discarded only while nothing has been added to it.
- A global hotkey pressed while the main window is focused is handled by the main window.
- Accent blue means only the primary action and selection; done uses its own done-lamp colour; destructive actions use the danger colour.
- Sizes use 1rem = 14px.

## From the flowchart review

1. **Trash entry point:** a Trash icon button in the title bar on both versions, as well as the command palette's Open Trash.
2. **Desktop window controls:** native OS controls on every desktop platform; the app draws none. The web draws none either; the browser owns the window.
3. **Search beyond this list (redesigned):** when the search scope is Whole project or All projects, the pane shows the matches grouped by list, each group headed by its list (and project) name. Choosing a match opens that list with the task selected and previewed. The match count counts every group. This replaces the reference, where the scope changed only the count.
4. **Empty "Add note" box:** kept, as the dashed variant of Button, used for every empty "add something here" slot (notes, attachments), so the exception is one documented variant rather than a separate widget.
5. **Draft's Screenshot slot on the web:** replaced by **Image**: paste an image (Ctrl/Cmd+V) into the draft, or pick a file. It creates the task (as "Untitled" if it has no title) with the image attached, uploaded to the server. Desktop keeps Screenshot.
6. **Bulk actions:** one place only, the bulk bar. The multi-select inspector shows the selection (count and the selected tasks) with Open and Clear selection, and no verb grid. The same job no longer has two widgets.
7. **Opening the command palette on the web:** both a visible control (a search-and-commands button in the title bar) and Ctrl/Cmd+K, captured while the page has focus (pages can take this chord from the browser).
8. **Trash on desktop:** opens in its own window, as a desktop app would (Finder's Trash is a window). The web keeps it in the side panel.
9. **Who may rename or delete a project on the web:** projects belong to one owner, and only the owner may rename or delete them. Sharing projects between users is out of scope for the first web version.
10. **Enter on destructive confirmations:** Enter does not confirm. Delete list, Delete project, Move to Trash, Empty Trash and Delete permanently need a click on the danger button, or Tab to it and Space. Escape cancels. Prompt sheets (new list, rename…) keep Enter to accept.
11. **Undo fails on the web:** the toast stays, shows the error, and Undo becomes "Try again". Dismiss still closes it.
12. **Delete permanently in Trash:** immediate for a single row. Purging several items at once, and Empty Trash, ask first, naming how many.
13. **Default focus in confirmations:** Cancel, on both versions (it follows from 10).
14. **Trash restore or purge fails on the web, or the item changed elsewhere:** the Trash reloads its list and shows a short inline message at the top of the pane saying what happened. (Undo failures follow 11.)
15. **Shortcuts on the web:** the Global group is removed (a browser cannot have global hotkeys). In-app shortcuts stay and can be changed; the recorder refuses chords the browser keeps for itself and says so.
16. **Reduced motion:** a three-way choice like Theme: System (follow the OS), On, Off. On both versions.
17. **Where settings are kept on the web:** Theme, Reduced motion and Show completed tasks are stored per account, so they follow the user across devices. Desktop keeps them locally.
18. **Browser-reserved chords on the web:** remapped to web defaults. New task Alt+N; New list Alt+Shift+N; Search "/" when no text field has focus; Switch to list 1–9 Alt+1…9; Previous and next list tab Alt+Shift+Left and Right; Previous and next project Alt+Shift+[ and ]. Desktop keeps the reference defaults.
19. **Desktop Settings window size:** opens at 640 × 520 and can be resized, never smaller than 640 × 520 (as the reference code does).
20. **Capture bar, Cmd+Enter (desktop only):** while the bar has focus, Cmd+Enter (Ctrl+Enter on Windows) submits from the title or the notes; the global show/hide toggle acts only when the bar does not have focus. Escape dismisses the bar.
