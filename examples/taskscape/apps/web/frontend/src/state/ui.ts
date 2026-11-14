// What the user is looking at, as opposed to the data (which is the API's, cached by React Query).
import { create } from "zustand";
import type { Id } from "../api/types";

export type Aside = "preview" | "trash" | "filter" | "none";

export interface UiState {
  projectId: Id | null;
  /** The active list of each pane; two panes in split view. */
  panes: (Id | null)[];
  activePane: number;
  /** Selected task ids (the bulk bar shows when there are any). */
  selection: Id[];
  /** The task shown in the inspector. */
  previewId: Id | null;
  aside: Aside;
  paletteOpen: boolean;
  settingsOpen: boolean;
  setProject(id: Id | null): void;
  openList(id: Id | null, pane?: number): void;
  split(on: boolean): void;
  select(ids: Id[]): void;
  preview(id: Id | null): void;
  setAside(a: Aside): void;
  setPalette(open: boolean): void;
  setSettings(open: boolean): void;
}

export const useUi = create<UiState>((set) => ({
  projectId: null,
  panes: [null],
  activePane: 0,
  selection: [],
  previewId: null,
  aside: "preview",
  paletteOpen: false,
  settingsOpen: false,
  setProject: (projectId) => set({ projectId, panes: [null], activePane: 0, selection: [], previewId: null }),
  openList: (id, pane) => set((s) => {
    const i = pane ?? s.activePane;
    const panes = [...s.panes];
    panes[i] = id;
    return { panes, activePane: i, selection: [] };
  }),
  split: (on) => set((s) => (on ? { panes: [s.panes[0], s.panes[1] ?? s.panes[0]], activePane: 1 } : { panes: [s.panes[s.activePane]], activePane: 0 })),
  select: (selection) => set({ selection }),
  preview: (previewId) => set({ previewId }),
  setAside: (aside) => set({ aside }),
  setPalette: (paletteOpen) => set({ paletteOpen }),
  setSettings: (settingsOpen) => set({ settingsOpen }),
}));
