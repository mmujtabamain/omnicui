import { useState } from "react";
import * as TabsPrimitive from "@radix-ui/react-tabs";
import { SidebarTab } from "../components/ui/sidebar-tab";

/** A root value no pane can have, so an unselected tab always differs from it, even with an empty label. */
const NO_PANE = "\u0000none";

/**
 * A vertical tablist holding one SidebarTab. Activation is manual: focus only moves, and Enter or
 * Space selects, so a focus alone never emits select.
 */
function SidebarTabHarness({ emit, icon = "", label = "", selected = false, scoped = true, dimmed = false, badge = 0 }: any) {
  const [current, setCurrent] = useState<boolean>(selected);

  return (
    <TabsPrimitive.Root
      orientation="vertical"
      activationMode="manual"
      value={current ? label : NO_PANE}
      onValueChange={() => {
        if (current) return;
        setCurrent(true);
        emit("select", { pane: label });
      }}
    >
      <TabsPrimitive.List aria-label="Settings panes" className="flex w-[9.929rem] max-w-full flex-col">
        <SidebarTab icon={icon} label={label} selected={current} scoped={scoped} dimmed={dimmed} badge={badge} />
      </TabsPrimitive.List>
    </TabsPrimitive.Root>
  );
}

export default { id: "ui.sidebartab", Component: SidebarTabHarness, events: ["select"] };
