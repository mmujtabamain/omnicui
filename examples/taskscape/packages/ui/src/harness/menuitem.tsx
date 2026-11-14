import { useEffect, useRef, useState } from "react";
import * as DropdownMenuPrimitive from "@radix-ui/react-dropdown-menu";
import { MenuItem } from "../components/ui/menu-item";

/** Demo lists a submenu row offers. */
const LISTS = ["Inbox", "Today", "Someday"];

const SIBLING_CLASS =
  "flex h-[2.143rem] items-center px-space-6 text-[0.929rem] text-content-1 outline-none data-[disabled]:opacity-40 data-[highlighted]:bg-wash-2";

/**
 * The row sits in a Menu panel, as it does in the app (Radix mounts rows only inside one). The panel is
 * held open and the row takes focus on open, so the test drives it. Plain Radix rows around it are the
 * harness-only siblings for the focus and arrow-key tests.
 */
function MenuItemHarness({ emit, selection = "action", checked = false, value = "", label = "Item", rowActions = false, ...props }: any) {
  const radio = selection === "radio";
  const [group, setGroup] = useState<string>(checked ? value : "");
  const rowRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    // Project rows rest with their actions hidden; focus-within would reveal them, so only the other rows take focus here.
    if (!rowActions) rowRef.current?.focus();
  }, []);

  const main = (
    <MenuItem
      ref={rowRef}
      selection={selection}
      value={value}
      label={label}
      rowActions={rowActions}
      {...props}
      onSelect={() => emit("activated", { value: radio ? value : "", label })}
      onSubmenuToggle={(open: boolean) => emit("submenuToggled", { open })}
      onRenameAction={() => emit("renamePressed", { label })}
      onDeleteAction={() => emit("deletePressed", { label })}
      submenu={LISTS.map((name) => (
        <div key={name} role="menuitem" tabIndex={-1} className={SIBLING_CLASS.replace("text-content-1", "text-content-2")}>
          {name}
        </div>
      ))}
    />
  );

  const sibling = (slot: string, text: string, rowValue: string, disabled = false) =>
    radio ? (
      <DropdownMenuPrimitive.RadioItem key={slot} data-slot={slot} value={rowValue} disabled={disabled} textValue={text} className={SIBLING_CLASS}>
        {text}
      </DropdownMenuPrimitive.RadioItem>
    ) : (
      <DropdownMenuPrimitive.Item key={slot} data-slot={slot} disabled={disabled} textValue={text} className={SIBLING_CLASS}>
        {text}
      </DropdownMenuPrimitive.Item>
    );

  const rows = (
    <>
      {sibling("previous", "Paste", "paste")}
      {main}
      {sibling("disabledSibling", "Archived lists", "archived", true)}
      {sibling("sibling", radio ? "Last 30 days" : "Move to list", "30d")}
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
        className="z-(--z-index-menu) flex min-w-[12rem] w-[16rem] flex-col rounded-(--radius-panel) border border-edge-1 bg-surface-3 p-space-5 shadow-menu outline-none"
      >
        {radio ? (
          <DropdownMenuPrimitive.RadioGroup
            aria-label="Date range"
            data-slot="group"
            value={group}
            onValueChange={setGroup}
            className="flex flex-col"
          >
            {rows}
          </DropdownMenuPrimitive.RadioGroup>
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
