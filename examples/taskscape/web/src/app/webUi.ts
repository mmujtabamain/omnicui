// What the web shell has open that the desktop app opens as windows or
// native panels: Settings (a dialog in the page), the quick-capture bar (the
// tray's capture bar, as a bar in the page), and, in the narrow layout, the
// details drawer over the pane.
import { create } from 'zustand';

interface WebUiState {
  settingsOpen: boolean;
  captureOpen: boolean;
  setSettingsOpen: (open: boolean) => void;
  setCaptureOpen: (open: boolean) => void;
}

export const useWebUi = create<WebUiState>((set) => ({
  settingsOpen: false,
  captureOpen: false,
  setSettingsOpen: (settingsOpen) => set({ settingsOpen }),
  setCaptureOpen: (captureOpen) => set({ captureOpen }),
}));
