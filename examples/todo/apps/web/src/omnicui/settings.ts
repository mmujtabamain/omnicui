// Generated from examples/todo/omnicui/contracts/settings.contract.yaml by stubs.ts. Do not edit: change the contract instead.

export interface SettingsThemeChanged { theme: "system" | "light" | "dark"; }
export interface SettingsShowCompletedChanged { show: boolean; }
export interface SettingsClosed {  }
export interface SettingsProps {
  theme?: "system" | "light" | "dark";
  showCompleted?: boolean;
  onThemeChanged?: (event: SettingsThemeChanged) => void;
  onShowCompletedChanged?: (event: SettingsShowCompletedChanged) => void;
  onClosed?: (event: SettingsClosed) => void;
}
export const SettingsDefaults = { theme: "system", showCompleted: true } as const;
export const Parts = { themeSystem: "todo.settings.themeSystem", themeLight: "todo.settings.themeLight", themeDark: "todo.settings.themeDark", showCompleted: "todo.settings.showCompleted", settingsBack: "todo.settings.settingsBack" } as const;
export const Events = ["themeChanged", "showCompletedChanged", "closed"] as const;
