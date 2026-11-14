import * as React from "react";
import { Command } from "cmdk";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/** rows: the groups and their rows; empty: only the empty message, the list is not drawn. */
const commandListVariants = cva("flex min-h-0 min-w-0 w-full flex-col text-content-1", {
  variants: {
    content: {
      rows: "",
      empty: "",
    },
  },
  defaultVariants: { content: "rows" },
});

export interface CommandListRow {
  id: string;
  label: string;
}

export interface CommandListGroup {
  heading: string;
  rows: CommandListRow[];
}

export interface CommandListProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "content" | "defaultValue"> {
  /** Populated groups and rows, or the empty message alone. */
  content?: VariantProps<typeof commandListVariants>["content"];
  /** Group headings and rows. */
  groups?: CommandListGroup[];
  /** The active row id; scrolled into view with block nearest. "" means no active row. */
  activeId?: string;
  /** Shown when content is empty. */
  emptyMessage?: string;
  /** Runs a command: the id of the row that was pressed. */
  onPick?: (id: string) => void;
}

const NO_GROUPS: CommandListGroup[] = [];

const reducedMotion = () => {
  const root = document.documentElement;
  if (root.classList.contains("reduced-motion")) return true;
  return !root.classList.contains("motion-on") && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
};

/**
 * The palette results list (cmdk over a vertical scroll region). Focus stays in the palette input,
 * so rows are options that never take DOM focus; the active row is passed in as activeId.
 */
export const CommandList = React.forwardRef<HTMLDivElement, CommandListProps>(function CommandList(
  { content = "rows", groups = NO_GROUPS, activeId = "", emptyMessage = "No results", onPick, className, ...props },
  ref,
) {
  const listRef = React.useRef<HTMLDivElement>(null);
  const headingBase = React.useId();

  const firstId = groups[0]?.rows[0]?.id;
  const lastGroup = groups[groups.length - 1];
  const lastId = lastGroup?.rows[lastGroup.rows.length - 1]?.id;

  // Keep the active row in view (block nearest); instant when reduced motion is on.
  React.useEffect(() => {
    const list = listRef.current;
    if (!list || !activeId || content === "empty") return;
    const row = Array.from(list.querySelectorAll<HTMLElement>("[data-row-id]")).find(
      (el) => el.dataset.rowId === activeId,
    );
    row?.scrollIntoView({ block: "nearest", behavior: reducedMotion() ? "auto" : "smooth" });
  }, [activeId, content, groups]);

  return (
    <Command
      ref={ref}
      shouldFilter={false}
      value={activeId}
      disablePointerSelection
      className={cn(commandListVariants({ content }), className)}
      {...props}
    >
      {content === "empty" ? (
        <div
          data-slot="empty"
          role="status"
          aria-live="polite"
          className="px-3 py-6 text-center text-[0.929rem] text-content-2"
        >
          {emptyMessage}
        </div>
      ) : (
        <Command.List
          ref={listRef}
          data-slot="list"
          className="max-h-[60vh] min-h-0 flex-1 overflow-y-auto overscroll-contain py-3 outline-none"
        >
          {groups.map((group, index) => {
            const headingId = `${headingBase}-${index}`;
            const first = index === 0;
            return (
              <div
                key={`${index}-${group.heading}`}
                role="group"
                aria-labelledby={headingId}
                data-slot={first ? "firstGroup" : undefined}
                className="flex flex-col"
              >
                <h3
                  id={headingId}
                  data-slot={first ? "firstGroupHeading" : undefined}
                  className="px-3.5 pt-2 pb-1 text-[0.786rem] leading-[1.1] font-medium text-content-2"
                >
                  {group.heading}
                </h3>
                {group.rows.map((row) => (
                  <Command.Item
                    key={row.id}
                    value={row.id}
                    onSelect={() => onPick?.(row.id)}
                    data-row-id={row.id}
                    data-slot={row.id === firstId ? "firstRow" : row.id === lastId ? "lastRow" : undefined}
                    className="mx-1.5 flex h-[2rem] min-w-0 cursor-default items-center rounded-(--radius-field) px-2 text-[0.929rem] text-content-1 outline-none select-none hover:bg-wash-1 data-[selected=true]:bg-selection-1"
                  >
                    <span className="min-w-0 truncate">{row.label}</span>
                  </Command.Item>
                ))}
              </div>
            );
          })}
        </Command.List>
      )}
    </Command>
  );
});

export { commandListVariants };
