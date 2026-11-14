import { useState, type CSSProperties } from "react";
import * as DropdownMenuPrimitive from "@radix-ui/react-dropdown-menu";
import { MenuItem } from "../components/ui/menu-item";

const noop = () => {};

/**
 * Style for the harness-only rows the tests press, focus and arrow to. Each is a real box, so a press reaches it,
 * but nothing is drawn: opacity 0, placed below the panel (the Content is positioned), one slot per row so no two overlap.
 */
function unseen(slot: number): CSSProperties {
  return { position: "absolute", left: 0, right: 0, top: `calc(100% + ${slot * 2}rem)`, height: "1.857rem", opacity: 0 };
}

/** The row is mounted in an open menu, as the product does; Radix needs its Menu around the row. */
function MenuItemHarness({ emit, selection = "action", trailing = "none", checked = false, value = "", label = "Item", ...props }: any) {
  const isRadio = selection === "radio";
  const rowValue = isRadio ? String(value) : "";
  const rowLabel = String(label);
  // The group's chosen radio value. The row's checked prop seeds it, so a press on a sibling unchecks the row.
  const [chosen, setChosen] = useState<string | null>(isRadio && checked ? rowValue : null);

  // Submenu rows carry their own data-slot, so the row part is only the row under test.
  const submenu = (
    <>
      <MenuItem data-slot="submenuItem" label="Inbox" />
      <MenuItem data-slot="submenuItem" label="Today" />
      <MenuItem data-slot="submenuItem" label="Someday" />
    </>
  );

  // Harness-only siblings are plain Radix items, so their text is not a second label part of the row.
  const sibling = (slot: string, index: number, text: string, disabled: boolean) => (
    <DropdownMenuPrimitive.Item data-slot={slot} disabled={disabled} onSelect={noop} style={unseen(index)}>
      {text}
    </DropdownMenuPrimitive.Item>
  );
  const radioSibling = (slot: string, index: number, text: string, optionValue: string, disabled = false) => (
    <DropdownMenuPrimitive.RadioGroup key={slot} value={chosen === optionValue ? "on" : "off"} onValueChange={noop}>
      <DropdownMenuPrimitive.RadioItem
        data-slot={slot}
        value="on"
        disabled={disabled}
        onSelect={() => setChosen(optionValue)}
        style={unseen(index)}
      >
        {text}
      </DropdownMenuPrimitive.RadioItem>
    </DropdownMenuPrimitive.RadioGroup>
  );

  const rows = (
    <>
      {isRadio ? radioSibling("previous", 0, "Last 90 days", "90d") : sibling("previous", 0, "Paste", false)}
      <MenuItem
        {...props}
        selection={selection}
        trailing={trailing}
        label={label}
        value={rowValue}
        checked={isRadio && chosen === rowValue}
        submenu={trailing === "submenu" ? submenu : undefined}
        onSelect={() => {
          if (isRadio) setChosen(rowValue);
          emit("activated", { value: rowValue, label: rowLabel });
        }}
        onSubmenuToggle={(open: boolean) => emit("submenuToggled", { open })}
        onRename={() => emit("renamePressed", { label: rowLabel })}
        onDelete={() => emit("deletePressed", { label: rowLabel })}
      />
      {isRadio
        ? radioSibling("disabledSibling", 1, "Last 14 days", "14d", true)
        : sibling("disabledSibling", 1, "Archived lists", true)}
      {isRadio ? radioSibling("sibling", 2, "Last 30 days", "30d") : sibling("sibling", 2, "Move to list", false)}
    </>
  );

  return (
    <DropdownMenuPrimitive.Root open modal={false} onOpenChange={noop}>
      <DropdownMenuPrimitive.Trigger asChild>
        <div className="block h-0" />
      </DropdownMenuPrimitive.Trigger>
      <DropdownMenuPrimitive.Content
        side="bottom"
        align="start"
        sideOffset={4}
        onCloseAutoFocus={(event) => event.preventDefault()}
        className="relative z-(--z-index-menu) w-[17.5rem] rounded-(--radius-panel) bg-surface-3 p-space-5 shadow-menu outline-none"
      >
        {isRadio ? (
          <div role="group" aria-label="Date range" data-slot="group">
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
