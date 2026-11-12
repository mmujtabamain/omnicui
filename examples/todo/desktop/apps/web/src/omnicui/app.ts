// Generated from examples/todo/desktop/omnicui/contracts/app.contract.yaml by stubs.ts. Do not edit: change the contract instead.

export interface AppNavigated { screen: "list" | "detail" | "settings"; id: string; }
export interface AppOpened { id: string; }
export interface AppAdded { id: string; title: string; }
export interface AppToggled { id: string; done: boolean; }
export interface AppRenamed { id: string; title: string; }
export interface AppDeleted { id: string; }
export interface AppThemeChanged { theme: "system" | "light" | "dark"; }
export interface AppShowCompletedChanged { show: boolean; }
export interface AppProps {
  seed?: "empty" | "three" | "many";
  onNavigated?: (event: AppNavigated) => void;
  onOpened?: (event: AppOpened) => void;
  onAdded?: (event: AppAdded) => void;
  onToggled?: (event: AppToggled) => void;
  onRenamed?: (event: AppRenamed) => void;
  onDeleted?: (event: AppDeleted) => void;
  onThemeChanged?: (event: AppThemeChanged) => void;
  onShowCompletedChanged?: (event: AppShowCompletedChanged) => void;
}
export const AppDefaults = { seed: "three" } as const;
export const Parts = { openSettings: "todo.app.openSettings", tasksHeading: "todo.app.tasksHeading", left: "todo.app.left", add: "todo.app.add", newTitle: "todo.app.newTitle", confirmAdd: "todo.app.confirmAdd", row1: "todo.app.row1", row2: "todo.app.row2", row1Check: "todo.app.row1Check", title: "todo.app.title", done: "todo.app.done", delete: "todo.app.delete", detailBack: "todo.app.detailBack", themeSystem: "todo.app.themeSystem", themeDark: "todo.app.themeDark", showCompleted: "todo.app.showCompleted", settingsBack: "todo.app.settingsBack" } as const;
export const Events = ["navigated", "opened", "added", "toggled", "renamed", "deleted", "themeChanged", "showCompletedChanged"] as const;
