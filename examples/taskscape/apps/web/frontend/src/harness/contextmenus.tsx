// The harness entry for app.contextmenus: the whole main window, signed in as the demo account. The dataset
// is loaded before this mounts; the window opens on the first project and its first list.
import { useLayoutEffect } from "react";
import { TooltipProvider } from "@taskscape/ui/tooltip";
import { AppHarness } from "./lib/AppHarness";
import { MainWindow } from "../screens/main-window/MainWindow";
import { observeContextMenuParts } from "../screens/main-window/contextmenu-parts";

/** Starts the context-menu tagger before the main window's own tagger (layout effects run before passive ones). */
function ContextMenuParts() {
  useLayoutEffect(() => observeContextMenuParts(), []);
  return null;
}

function ContextMenusHarness(_props: { dataset?: string }) {
  return (
    <AppHarness>
      <TooltipProvider>
        <ContextMenuParts />
        <MainWindow />
      </TooltipProvider>
    </AppHarness>
  );
}

export default { id: "app.contextmenus", Component: ContextMenusHarness, events: [] };
