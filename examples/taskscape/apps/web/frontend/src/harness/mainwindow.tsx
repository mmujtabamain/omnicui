// The harness entry for app.mainwindow: the whole window, signed in as the demo account. The dataset
// is loaded before this mounts; the window opens on the first project and its first list.
import { TooltipProvider } from "@taskscape/ui/tooltip";
import { AppHarness } from "./lib/AppHarness";
import { MainWindow } from "../screens/main-window/MainWindow";

function MainWindowHarness(_props: { dataset?: string }) {
  return (
    <AppHarness>
      <TooltipProvider>
        <MainWindow />
      </TooltipProvider>
    </AppHarness>
  );
}

export default { id: "app.mainwindow", Component: MainWindowHarness, events: [] };
