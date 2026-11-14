import { useState } from "react";
import * as DropdownMenuPrimitive from "@radix-ui/react-dropdown-menu";
import { ListRow } from "../components/ui/list-row";

function ListRowHarness({ emit, host = "list", title, ...props }: any) {
  // The project switcher closes after a choice; the menu harness owns its open state.
  const [open, setOpen] = useState(true);
  const row = (
    <ListRow
      {...props}
      title={title}
      host={host}
      onActivate={() => emit("activated", { title })}
      onAction={(action: string) => emit("actionRequested", { action })}
    />
  );
  if (host !== "menu") return <div role="list" className="w-full p-space-5">{row}</div>;
  return (
    <DropdownMenuPrimitive.Root open={open} onOpenChange={setOpen} modal={false}>
      <DropdownMenuPrimitive.Trigger asChild>
        <span aria-hidden="true" className="block h-0 w-full" />
      </DropdownMenuPrimitive.Trigger>
      <DropdownMenuPrimitive.Portal>
        <DropdownMenuPrimitive.Content
          sideOffset={4}
          className="z-(--z-index-menu) w-[17.5rem] rounded-(--radius-panel) bg-surface-3 p-space-4 shadow-menu"
        >
          {row}
        </DropdownMenuPrimitive.Content>
      </DropdownMenuPrimitive.Portal>
    </DropdownMenuPrimitive.Root>
  );
}

export default { id: "ui.listrow", Component: ListRowHarness, events: ["activated", "actionRequested"] };
