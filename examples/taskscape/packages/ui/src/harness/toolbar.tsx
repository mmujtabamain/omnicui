import { Button } from "../components/ui/button";
import { Toolbar, ToolbarItem } from "../components/ui/toolbar";

/** Demo actions per placement: three buttons in each. */
const ACTIONS: Record<string, { icon: string; label: string }[]> = {
  inline: [
    { icon: "search", label: "Search" },
    { icon: "filter_list", label: "Filter" },
    { icon: "sort", label: "Sort" },
  ],
  strip: [
    { icon: "restore_from_trash", label: "Restore" },
    { icon: "drive_file_move", label: "Move" },
    { icon: "delete_forever", label: "Delete forever" },
  ],
  footer: [
    { icon: "drive_file_move", label: "Move" },
    { icon: "label", label: "Tag" },
    { icon: "delete", label: "Delete" },
  ],
};

const SLOTS = ["first", "second", "third"];

function ToolbarHarness({ emit: _emit, placement = "inline", ariaLabel, count, ...props }: any) {
  const actions = ACTIONS[placement] ?? ACTIONS.inline;
  return (
    <Toolbar placement={placement} ariaLabel={ariaLabel} count={count} {...props}>
      {actions.map((action, i) => (
        <ToolbarItem asChild key={SLOTS[i]}>
          <Button
            data-slot={SLOTS[i]}
            icon={action.icon}
            ariaLabel={action.label}
            description={action.label}
            shape="square"
            variant="ghost"
            size="md"
          />
        </ToolbarItem>
      ))}
    </Toolbar>
  );
}

export default { id: "ui.toolbar", Component: ToolbarHarness, events: [] };
