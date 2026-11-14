// The sheet queue and the toast that Move to Trash raises. One sheet is up at a time: a new one
// dismisses the old. Each sheet answers once, through the promise openSheet returned.
import { create } from "zustand";
import type { Project, Task, TaskList } from "../../api/types";

/** The sheet to show. Built by the caller from the app's own spec for that action. */
export type SheetSpec =
  | { kind: "newList"; projectId: number }
  | { kind: "newProject" }
  | { kind: "renameList"; list: TaskList }
  | { kind: "renameProject"; project: Project }
  | { kind: "deleteList"; list: TaskList }
  | { kind: "deleteProject"; project: Project }
  | { kind: "deleteConfirm"; task: Task };

/** How the sheet resolved. text is the accepted name (prompts); id is the instead id. */
export interface SheetResult {
  action: "accept" | "dismiss" | "instead";
  text?: string;
  id?: string;
}

/** The toast after a Move to Trash. retry runs Undo, and again after a failure ("Try again"). */
export interface ToastState {
  message: string;
  failed: boolean;
  errorMessage: string;
  retry?: () => Promise<unknown>;
}

interface Pending {
  key: number;
  spec: SheetSpec;
  settle: (result: SheetResult) => void;
}

interface SheetsState {
  pending: Pending | null;
  toast: ToastState | null;
  open(spec: SheetSpec): Promise<SheetResult>;
  close(result: SheetResult): void;
  setToast(toast: ToastState | null): void;
}

let nextKey = 0;

export const useSheets = create<SheetsState>((set, get) => ({
  pending: null,
  toast: null,
  open: (spec) =>
    new Promise<SheetResult>((settle) => {
      // Only one sheet at a time: the one already up is dismissed.
      get().pending?.settle({ action: "dismiss" });
      set({ pending: { key: ++nextKey, spec, settle } });
    }),
  close: (result) => {
    const pending = get().pending;
    if (!pending) return;
    set({ pending: null });
    pending.settle(result);
  },
  setToast: (toast) => set({ toast }),
}));

/** Shows a sheet over the window and resolves with its answer. */
export function openSheet(spec: SheetSpec): Promise<SheetResult> {
  return useSheets.getState().open(spec);
}
