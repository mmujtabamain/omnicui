import { AppHarness } from "./lib/AppHarness";
import { SettingsDialog } from "../screens/settings/SettingsDialog";

function SettingsGeneralHarness() {
  return <AppHarness path="/settings"><SettingsDialog /></AppHarness>;
}

export default { id: "app.settingsgeneral", Component: SettingsGeneralHarness, events: [] };
