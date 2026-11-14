/**
 * Toast: the single-slot undo notice pinned bottom-centre of the main window.
 *
 * Mount <ToastHost /> once in the main window. It renders nothing until a notice is set.
 * Show an undoable action's notice from anywhere with showUndoToast:
 *
 *   showUndoToast("Moved to Trash", async () => {
 *     await ep.trash.restore(trashedIds);
 *     return { count: 1, title: task.title };   // what the Undo restored
 *   });
 *
 * The message is the text shown ("Moved to Trash", or "<n> moved to Trash" for a bulk delete with
 * n root tasks). The undo runs when the user presses Undo; a rejected undo keeps the notice open
 * with the error and turns the button into "Try again". The notice closes by itself after 6 s,
 * a newer notice replaces it, and Dismiss or a successful Undo closes it.
 */
export { ToastHost } from "./ToastHost";
export type { ToastHostProps } from "./ToastHost";
export { showUndoToast, dismissToast, resetToast, useToastStore } from "./store";
export type { Notice, Restored, UndoFn } from "./store";
