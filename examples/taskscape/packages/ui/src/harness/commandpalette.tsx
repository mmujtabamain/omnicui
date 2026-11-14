import { useMemo, useState } from "react";
import { CommandPalette, type PaletteCommand } from "../components/ui/command-palette";

/** 13 rows: 9 plain (30px) and 4 with a shortcut (32px). Four match "trash": 1 plain, 3 with a shortcut. */
const ALL: PaletteCommand[] = [
  { id: "create-task", label: "Create task", icon: "add", shortcut: "Mod+N" },
  { id: "create-project", label: "Create project", icon: "create_new_folder" },
  { id: "go-today", label: "Go to today", icon: "today" },
  { id: "go-inbox", label: "Go to inbox", icon: "inbox" },
  { id: "go-projects", label: "Go to projects", icon: "folder" },
  { id: "go-settings", label: "Go to settings", icon: "settings" },
  { id: "switch-theme", label: "Switch theme", icon: "dark_mode" },
  { id: "toggle-sidebar", label: "Toggle sidebar", icon: "left_panel_close" },
  { id: "sign-out", label: "Sign out", icon: "logout" },
  { id: "open-trash", label: "Open Trash", icon: "delete" },
  { id: "empty-trash", label: "Empty Trash", icon: "delete_forever", shortcut: "Mod+Shift+Backspace" },
  { id: "restore-trash", label: "Restore from Trash", icon: "restore_from_trash", shortcut: "Mod+Shift+R" },
  { id: "move-trash", label: "Move project to Trash", icon: "delete", shortcut: "Mod+Delete" },
];

/** More rows than the card shows, so the list scrolls. */
const MANY: PaletteCommand[] = Array.from({ length: 40 }, (_, i) => ({
  id: `list-${i + 1}`,
  label: `Go to list ${i + 1}`,
  icon: "list",
}));

function CommandPaletteHarness({ emit, commandSet = "all", open = true, ...props }: any) {
  const [isOpen, setOpen] = useState<boolean>(open);
  const commands = useMemo(() => (commandSet === "many" ? MANY : ALL), [commandSet]);
  return (
    <CommandPalette
      {...props}
      open={isOpen}
      commands={commands}
      onActiveChange={(index: number) => emit("activeChange", { index })}
      onSelect={(index: number) => emit("select", { index })}
      onClose={(reason: string) => {
        emit("close", { reason });
        setOpen(false);
      }}
    />
  );
}

export default {
  id: "ui.commandpalette",
  Component: CommandPaletteHarness,
  events: ["activeChange", "select", "close"],
};
