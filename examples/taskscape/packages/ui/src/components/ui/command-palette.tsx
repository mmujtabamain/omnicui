import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { CommandInput } from "./command-input";
import { ComboboxItem } from "./combobox-item";
import { Overlay } from "./overlay";

/** One palette row. value is the text the filter matches; the label is used when it is empty. */
export interface PaletteCommand {
  id: string;
  label: string;
  value?: string;
  icon?: string;
  shortcut?: string;
}

/**
 * The card. overlay: 490px wide (35rem at 14px, min with the viewport), top edge at 91px (6.5rem),
 * at most 60vh tall; its height follows the rows. window: the same card centred in the window.
 */
export const commandPaletteVariants = cva("flex min-w-0 max-h-[60vh] w-[min(35rem,100%)] flex-col overflow-hidden", {
  variants: {
    presentation: {
      overlay: "top-[6.5rem]",
      window: "",
    },
  },
  defaultVariants: { presentation: "overlay" },
});

export type CommandPaletteCloseReason = "escape" | "select";

export interface CommandPaletteProps {
  presentation?: "overlay" | "window";
  /** false: not rendered. */
  open?: boolean;
  /** Initial filter text. */
  query?: string;
  /** Starting highlight index. */
  activeIndex?: number;
  /** Accessible name of the dialog. */
  ariaLabel?: string;
  /** The palette rows, from getCommands. */
  commands?: PaletteCommand[];
  /** Arrow keys or pointer moved the highlight. */
  onActiveChange?: (index: number) => void;
  /** A row ran: Enter on the search, or a press on a row. */
  onSelect?: (index: number, command: PaletteCommand) => void;
  /** Escape ("escape") or a row ran ("select"). */
  onClose?: (reason: CommandPaletteCloseReason) => void;
  className?: string;
}

const NO_COMMANDS: PaletteCommand[] = [];

function filterCommands(commands: PaletteCommand[], query: string): PaletteCommand[] {
  const needle = query.trim().toLowerCase();
  if (needle === "") return commands;
  return commands.filter((command) => (command.value ?? command.label).toLowerCase().includes(needle));
}

function setOrRemove(el: Element, name: string, value: string) {
  if (value) el.setAttribute(name, value);
  else el.removeAttribute(name);
}

/** The part name a row carries: the first four rows and the last have their own, the rest are plain rows. */
function rowSlot(index: number, count: number): string {
  if (index === 0) return "firstRow";
  if (index === 1) return "secondRow";
  if (index === 2) return "thirdRow";
  if (index === count - 1) return "lastRow";
  return "row";
}

interface PaletteBodyProps {
  id: string;
  rows: PaletteCommand[];
  current: number;
  text: string;
  onText: (value: string) => void;
  onStep: (direction: "up" | "down") => void;
  onRun: (index: number) => void;
  onHighlight: (index: number) => void;
}

/**
 * What sits inside the dialog. It is its own component so its effects run once the dialog's
 * portal has mounted, when the search input exists.
 */
function PaletteBody({ id, rows, current, text, onText, onStep, onRun, onHighlight }: PaletteBodyProps) {
  const searchRef = React.useRef<HTMLDivElement>(null);
  const listId = `${id}-list`;
  const optionId = (index: number) => `${id}-option-${index}`;
  const hasRows = rows.length > 0;

  // The inner input is named "input" by InputGroup and CommandInput looks it up by that name, so it
  // is renamed "search" here once it exists, and its combobox wiring is kept current by this layout
  // effect (CommandInput's own wiring only runs when its props change, which they do not here).
  React.useLayoutEffect(() => {
    const field = searchRef.current?.querySelector<HTMLInputElement>("input");
    if (!field) return;
    field.setAttribute("data-slot", "search");
    field.setAttribute("aria-expanded", String(hasRows));
    setOrRemove(field, "aria-controls", hasRows ? listId : "");
    setOrRemove(field, "aria-activedescendant", hasRows ? optionId(current) : "");
  });

  React.useEffect(() => {
    searchRef.current?.querySelector<HTMLInputElement>("input")?.focus();
  }, []);

  // The active row is kept in view.
  React.useEffect(() => {
    if (!hasRows) return;
    document.getElementById(optionId(current))?.scrollIntoView({ block: "nearest" });
  }, [current, hasRows]);

  return (
    <>
      <CommandInput
        ref={searchRef}
        autoFocus={false}
        placeholder="Type a command or search"
        ariaLabel="Search commands"
        value={text}
        onValueChange={onText}
        onMoveActive={onStep}
        onSubmit={() => onRun(current)}
      />
      {hasRows ? (
        <div
          id={listId}
          role="listbox"
          data-slot="list"
          className="min-h-0 flex-1 overflow-y-auto overscroll-contain px-1.5 py-2 outline-none"
        >
          {rows.map((command, index) => (
            <ComboboxItem
              key={command.id}
              id={optionId(index)}
              data-slot={rowSlot(index, rows.length)}
              context="palette"
              icon={command.icon ?? ""}
              label={command.label}
              value={command.value ?? command.label}
              shortcut={command.shortcut ?? ""}
              active={index === current}
              onSelect={() => onRun(index)}
              onHighlight={() => onHighlight(index)}
            />
          ))}
        </div>
      ) : (
        <div
          data-slot="emptyMessage"
          role="status"
          aria-live="polite"
          className="px-3 py-[1.6rem] text-center text-[0.929rem] leading-[1.393rem] text-content-2"
        >
          No matching commands
        </div>
      )}
    </>
  );
}

/**
 * The command palette: a dialog card with a search input that keeps focus (a WAI-ARIA combobox
 * with a listbox popup). The highlight moves with the arrow keys and the pointer; Enter runs it.
 */
export const CommandPalette = React.forwardRef<HTMLDivElement, CommandPaletteProps>(function CommandPalette(
  {
    presentation = "overlay",
    open = true,
    query = "",
    activeIndex = 0,
    ariaLabel = "Command palette",
    commands = NO_COMMANDS,
    onActiveChange,
    onSelect,
    onClose,
    className,
  },
  ref,
) {
  const id = React.useId();
  const [text, setText] = React.useState(query);
  const [active, setActive] = React.useState(activeIndex);
  const rows = React.useMemo(() => filterCommands(commands, text), [commands, text]);
  const current = rows.length > 0 ? Math.min(active, rows.length - 1) : 0;

  const highlight = (next: number) => {
    if (next === current) return;
    setActive(next);
    onActiveChange?.(next);
  };

  const changeText = (value: string) => {
    setText(value);
    // A new filter highlights its first match.
    setActive(0);
    if (current !== 0) onActiveChange?.(0);
  };

  const step = (direction: "up" | "down") => {
    const last = Math.max(rows.length - 1, 0);
    highlight(Math.min(Math.max(current + (direction === "down" ? 1 : -1), 0), last));
  };

  const run = (index: number) => {
    const command = rows[index];
    if (!command) return;
    onSelect?.(index, command);
    onClose?.("select");
  };

  return (
    <Overlay
      ref={ref}
      open={open}
      placement={presentation === "window" ? "center" : "top"}
      scrim="dim"
      dismissOn="none"
      onDismiss={() => onClose?.("escape")}
      aria-label={ariaLabel}
      data-slot="panel"
      className={cn(commandPaletteVariants({ presentation }), className)}
    >
      <PaletteBody
        id={id}
        rows={rows}
        current={current}
        text={text}
        onText={changeText}
        onStep={step}
        onRun={run}
        onHighlight={highlight}
      />
    </Overlay>
  );
});
