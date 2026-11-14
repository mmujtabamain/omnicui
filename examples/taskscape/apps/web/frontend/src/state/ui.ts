// What the user is looking at, as opposed to the data (which is the API's, cached by React Query).
import { create } from "zustand";
import type { Id } from "../api/types";

export type Aside = "preview" | "trash" | "filter" | "none";

/** The preview panel width in px; its divider clamps to this range (F09-027). */
export const PREVIEW_MIN = 280;
export const PREVIEW_MAX = 420;
export const SPLIT_MIN = 0.25;
export const SPLIT_MAX = 0.75;

export interface UiState {
  projectId: Id | null;
  /** The active list of each pane; two panes in split view. */
  panes: (Id | null)[];
  activePane: number;
  /** Selected task ids (the bulk bar shows when there are any). */
  selection: Id[];
  /** The task shown in the inspector. */
  previewId: Id | null;
  /** A task whose inspector should open with a new note ready to type (the draft's Note button). */
  noteFor: Id | null;
  aside: Aside;
  paletteOpen: boolean;
  settingsOpen: boolean;
  /** The list a new-task draft is started in; null when no draft is open (at most one, F09-051). */
  draftListId: Id | null;
  /** The preview panel width in px, between PREVIEW_MIN and PREVIEW_MAX. */
  previewWidth: number;
  /** The split ratio of the left pane (split view), between SPLIT_MIN and SPLIT_MAX (F09-029). */
  splitRatio: number;
  setProject(id: Id | null): void;
  openList(id: Id | null, pane?: number): void;
  split(on: boolean): void;
  select(ids: Id[]): void;
  preview(id: Id | null): void;
  /** Shows the task and opens a new note on it. */
  previewWithNote(id: Id): void;
  setAside(a: Aside): void;
  setPalette(open: boolean): void;
  setSettings(open: boolean): void;
  /** Starts a draft in the list; it shows in the aside and no row is added until it is committed. */
  startDraft(listId: Id): void;
  setDraft(listId: Id | null): void;
  setPreviewWidth(px: number): void;
  setSplitRatio(ratio: number): void;
}

const clampWidth = (px: number) => Math.min(PREVIEW_MAX, Math.max(PREVIEW_MIN, px));
const clampRatio = (r: number) => Math.min(SPLIT_MAX, Math.max(SPLIT_MIN, r));

export const useUi = create<UiState>((set) => ({
  projectId: null,
  panes: [null],
  activePane: 0,
  selection: [],
  previewId: null,
  noteFor: null,
  aside: "preview",
  paletteOpen: false,
  settingsOpen: false,
  draftListId: null,
  previewWidth: 322,
  splitRatio: 0.5,
  setProject: (projectId) => set({ projectId, panes: [null], activePane: 0, selection: [], previewId: null, draftListId: null }),
  openList: (id, pane) => set((s) => {
    const i = pane ?? s.activePane;
    const panes = [...s.panes];
    panes[i] = id;
    return { panes, activePane: i, selection: [] };
  }),
  split: (on) => set((s) => (on ? { panes: [s.panes[0], s.panes[1] ?? s.panes[0]], activePane: 1 } : { panes: [s.panes[s.activePane]], activePane: 0 })),
  select: (selection) => set({ selection }),
  preview: (previewId) => set({ previewId }),
  previewWithNote: (id) => set({ previewId: id, noteFor: id, draftListId: null, selection: [], aside: "preview" }),
  setAside: (aside) => set({ aside }),
  setPalette: (paletteOpen) => set({ paletteOpen }),
  setSettings: (settingsOpen) => set({ settingsOpen }),
  startDraft: (listId) => set({ draftListId: listId, previewId: null, selection: [], aside: "preview" }),
  setDraft: (draftListId) => set({ draftListId }),
  setPreviewWidth: (px) => set({ previewWidth: clampWidth(px) }),
  setSplitRatio: (ratio) => set({ splitRatio: clampRatio(ratio) }),
}));
