// The single-slot toast store: one undo notice at a time, closed by Dismiss, by a successful Undo,
// or by a 6 s timer. A newer notice replaces the current one and restarts the timer.
import { create } from "zustand";

/** What an Undo restored: the number of root tasks and the title of the first one, as the API returns it. */
export interface Restored {
  count: number;
  title: string;
}

/** Runs the undo. It rejects when the server call fails; the notice then stays open. */
export type UndoFn = () => Promise<Restored>;

export interface Notice {
  id: number;
  message: string;
  undo: UndoFn;
  /** The last Undo failed: errorMessage shows and the action reads "Try again". */
  failed: boolean;
  errorMessage: string;
}

interface ToastState {
  notice: Notice | null;
}

export const useToastStore = create<ToastState>(() => ({ notice: null }));

/** The reference timer: 6000 ms, restarted by a newer notice, cancelled by Dismiss or Undo. */
const TIMEOUT_MS = 6000;

let seq = 0;
let timer: ReturnType<typeof setTimeout> | undefined;

function clearTimer() {
  if (timer !== undefined) {
    clearTimeout(timer);
    timer = undefined;
  }
}

function armTimer(id: number) {
  clearTimer();
  timer = setTimeout(() => {
    timer = undefined;
    if (useToastStore.getState().notice?.id === id) useToastStore.setState({ notice: null });
  }, TIMEOUT_MS);
}

/** Shows the notice, replacing any current one. */
export function showUndoToast(message: string, undo: UndoFn): void {
  const id = ++seq;
  useToastStore.setState({ notice: { id, message, undo, failed: false, errorMessage: "" } });
  armTimer(id);
}

/** Closes the notice if it is still the one with this id. Returns true when it closed. */
export function dismissToast(id: number): boolean {
  const current = useToastStore.getState().notice;
  if (!current || current.id !== id) return false;
  clearTimer();
  useToastStore.setState({ notice: null });
  return true;
}

/** A failed Undo: the notice stays open with the error, and no timer closes it. */
export function failToast(id: number, errorMessage: string): void {
  const current = useToastStore.getState().notice;
  if (!current || current.id !== id) return;
  clearTimer();
  useToastStore.setState({ notice: { ...current, failed: true, errorMessage } });
}

/** Clears any notice and its timer. */
export function resetToast(): void {
  clearTimer();
  useToastStore.setState({ notice: null });
}
