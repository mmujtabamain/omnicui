import { useState } from "react";
import * as DropdownMenuPrimitive from "@radix-ui/react-dropdown-menu";
import { MenuItem } from "../components/ui/menu-item";

/** Demo lists a submenu row offers. */
const LISTS = ["Inbox", "Today", "Someday"];

/**
 * The row sits in a Menu panel, as it does in the app (Radix mounts rows only inside one). The panel is
 * held open; the test drives it. Harness siblings sit around the row for the focus and arrow-key tests.
 */
function MenuItemHarness({ emit, selection = "action", checked = false, value = "", label = "Item", ...props }: any) {
  const radio = selection === "radio";
  const [group, setGroup] = useState<string>(checked ? value : "");

  const rowFor = (rowLabel: string, rowValue: string, extra: Record<string, unknown> = {}) => (
    <MenuItem
      selection={selection}
      label={rowLabel}
      value={rowValue}
      checked={radio && group === rowValue}
      onSelect={() => {
        if (radio) setGroup(rowValue);
        emit("activated", { value: radio ? rowValue : "", label: rowLabel });
      }}
      {...extra}
    />
  );

  const rows = (
    <>
      {rowFor("Paste", "paste", { "data-slot": "previous" })}
      {rowFor(
        label,
        value,
        {
          ...props,
          onSubmenuToggle: (open: boolean) => emit("submenuToggled", { open }),
          onRenameAction: () => emit("renamePressed", { label }),
          onDeleteAction: () => emit("deletePressed", { label }),
          submenu: LISTS.map((name) => (
            <div
              key={name}
              role="menuitem"
              tabIndex={-1}
              className="flex h-[2.143rem] items-center rounded-(--radius-field) px-space-6 text-[0.929rem] text-content-2"
            >
              {name}
            </div>
          )),
        },
      )}
      {rowFor("Archived lists", "", { disabled: true, "data-slot": "disabledSibling" })}
      {radio
        ? rowFor("Last 30 days", "30d", { "data-slot": "sibling" })
        : rowFor("Move to list", "", { "data-slot": "sibling" })}
    </>
  );

  return (
    <DropdownMenuPrimitive.Root open modal={false}>
      <DropdownMenuPrimitive.Trigger asChild>
        <span className="block h-px w-px" />
      </DropdownMenuPrimitive.Trigger>
      <DropdownMenuPrimitive.Content
        align="start"
        sideOffset={4}
        className="flex min-w-[12rem] w-[16rem] flex-col rounded-(--radius-panel) border border-edge-1 bg-surface-3 p-space-5 shadow-menu outline-none"
      >
        {radio ? (
          <div role="group" aria-label="Date range" data-slot="group" className="flex flex-col">
            {rows}
          </div>
        ) : (
          rows
        )}
      </DropdownMenuPrimitive.Content>
    </DropdownMenuPrimitive.Root>
  );
}

export default {
  id: "ui.menuitem",
  Component: MenuItemHarness,
  events: ["activated", "submenuToggled", "renamePressed", "deletePressed"],
};
