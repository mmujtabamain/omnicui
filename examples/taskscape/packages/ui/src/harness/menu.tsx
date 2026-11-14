import { useEffect, useState } from "react";
import { Button } from "../components/ui/button";
import { Menu, MenuSubmenuPanel, type MenuOpenReason } from "../components/ui/menu";
import { MenuItem } from "../components/ui/menu-item";
import { Separator } from "../components/ui/separator";

const PLACEMENTS = ["cursor", "control", "submenu", "anchored"];
const WIDTHS = ["content", "task", "tab", "project"];

function MenuHarness({
  emit,
  open: openProp = false,
  placement = "cursor",
  width = "content",
  offset = 4,
  pointX = 0,
  pointY = 0,
  closeOnSelect = true,
  ariaLabel = "Menu",
}: any) {
  if (!PLACEMENTS.includes(placement)) throw new Error(`Menu: placement must be one of ${PLACEMENTS.join(", ")}, got ${placement}`);
  if (!WIDTHS.includes(width)) throw new Error(`Menu: width must be one of ${WIDTHS.join(", ")}, got ${width}`);
  const [open, setOpen] = useState(Boolean(openProp));
  const [compact, setCompact] = useState(false);
  useEffect(() => setOpen(Boolean(openProp)), [openProp]);

  const changed = (next: boolean, reason: MenuOpenReason) => {
    setOpen(next);
    emit("openChange", { open: next, reason });
  };
  const pick = (id: string) => emit("pick", { id });

  return (
    <div className="flex items-start gap-space-7 p-space-8 text-content-1">
      <Menu
        open={open}
        onOpenChange={changed}
        placement={placement}
        width={width}
        offset={offset}
        pointX={pointX}
        pointY={pointY}
        closeOnSelect={closeOnSelect}
        ariaLabel={ariaLabel}
        trigger={<Button label="Actions" variant="outline" />}
      >
        <MenuItem data-slot="first" label="Open" onSelect={() => pick("first")} />
        <MenuItem data-slot="second" label="Rename" onSelect={() => pick("second")} />
        <Separator data-slot="separator" inset="menu" decorative={false} />
        <MenuItem data-slot="disabledRow" label="Delete" disabled />
        <MenuItem
          data-slot="submenu"
          label="Move to list"
          trailing="submenu"
          submenu={
            <MenuSubmenuPanel ariaLabel="Move to list">
              <MenuItem data-slot="submenuFirst" label="Inbox" onSelect={() => pick("submenuFirst")} />
            </MenuSubmenuPanel>
          }
        />
        <MenuItem
          data-slot="choice"
          selection="radio"
          value="compact"
          checked={compact}
          label="Compact"
          onSelect={() => {
            setCompact(true);
            pick("choice");
          }}
        />
        <MenuItem data-slot="last" label="Trash" onSelect={() => pick("last")} />
      </Menu>
      <Button data-slot="outside" label="Outside" variant="outline" />
    </div>
  );
}

export default { id: "ui.menu", Component: MenuHarness, events: ["openChange", "pick"] };
