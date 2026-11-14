// The Settings dialog on its Shortcuts tab (web, 640 x 520): the title bar, the sidebar (search and
// the three section tabs) and the Shortcuts pane with the Main window rows. Every change is saved as
// it is made, per account, and reported through onBindingChanged. Browser-reserved chords are refused
// while recording, and the web defaults use Alt chords (decision 18).
import * as React from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Badge } from "@taskscape/ui/badge";
import { Button } from "@taskscape/ui/button";
import { EmptyState } from "@taskscape/ui/empty-state";
import { Heading } from "@taskscape/ui/heading";
import { Icon } from "@taskscape/ui/icon";
import { Input } from "@taskscape/ui/input";
import { Keycap } from "@taskscape/ui/keycap";
import { Overlay } from "@taskscape/ui/overlay";
import { TabList, TabListItem } from "@taskscape/ui/tab-list";
import { Text } from "@taskscape/ui/text";
import { useParts } from "@taskscape/ui/lib/parts";
import { isMac } from "@taskscape/ui/lib/platform";
import * as ep from "../../api/endpoints";
import { keys, useShortcuts, useWrite } from "../../api/queries";
import type { Binding } from "../../api/types";

type PaneId = "shortcuts" | "general" | "about";

const PANES: { id: PaneId; label: string; icon: string; part: string }[] = [
  { id: "general", label: "General", icon: "tune", part: "tabGeneral" },
  { id: "shortcuts", label: "Shortcuts", icon: "keyboard", part: "tabShortcuts" },
  { id: "about", label: "About", icon: "info", part: "tabAbout" },
];

const INTRO = "Click a shortcut to change it — press the new keys, Backspace to remove, Esc to cancel.";
const GROUP = "Main window";

/** One shortcut row: the API's command id, its title, the web default and the parts it carries. */
interface MainRow {
  command: string;
  label: string;
  accel: string;
  row?: string;
  key?: string;
}

function row(command: string, label: string, accel: string, rowPart?: string, keyPart?: string): MainRow {
  return { command, label, accel, row: rowPart, key: keyPart };
}

/** The Main window group in catalog order (31 rows), with the web defaults (decision 18). */
const MAIN_ROWS: MainRow[] = [
  row("new_task", "New task", "Alt+N", "rowNewTask", "keycapNewTask"),
  row("search", "Search", "/", undefined, "keycapSearch"),
  row("toggle_preview", "Toggle preview", "Mod+\\"),
  row("open_settings", "Open settings", "Mod+,"),
  row("delete_task", "Delete task", "Mod+Backspace"),
  row("switch_list_1", "Switch to list 1", "Alt+1", undefined, "keycapSwitchList1"),
  row("switch_list_2", "Switch to list 2", "Alt+2"),
  row("switch_list_3", "Switch to list 3", "Alt+3"),
  row("switch_list_4", "Switch to list 4", "Alt+4"),
  row("switch_list_5", "Switch to list 5", "Alt+5"),
  row("switch_list_6", "Switch to list 6", "Alt+6"),
  row("switch_list_7", "Switch to list 7", "Alt+7"),
  row("switch_list_8", "Switch to list 8", "Alt+8"),
  row("switch_list_9", "Switch to list 9", "Alt+9"),
  row("prev_tab", "Previous list tab", "Alt+Shift+ArrowLeft", undefined, "keycapPrevTab"),
  row("next_tab", "Next list tab", "Alt+Shift+ArrowRight"),
  row("prev_project", "Previous project", "Alt+Shift+[", undefined, "keycapPrevProject"),
  row("next_project", "Next project", "Alt+Shift+]"),
  row("move_up", "Move task up", "Mod+Alt+ArrowUp"),
  row("move_down", "Move task down", "Mod+Alt+ArrowDown"),
  row("select_all", "Select all tasks", "Mod+A"),
  row("command_palette", "Command palette", "Mod+K"),
  row("toggle_split", "Toggle split view", "Mod+Alt+S"),
  row("focus_other_pane", "Focus other pane", "Mod+Alt+O"),
  row("new_list", "New list", "Alt+Shift+N", "rowNewList", "keycapNewList"),
  row("new_project", "New project", "Mod+Shift+P", "rowNewProject"),
  row("show_completed", "Show/hide completed", "Mod+Shift+H"),
  row("collapse_all", "Collapse all", "Mod+Alt+-"),
  row("expand_all", "Expand all", "Mod+Alt+="),
  row("undo", "Undo", "Mod+Z", "rowUndo", "keycapUndo"),
  row("redo", "Redo", "Mod+Shift+Z", "rowRedo", "keycapRedo"),
];

const DEFAULTS: Record<string, string> = Object.fromEntries(MAIN_ROWS.map((item) => [item.command, item.accel]));

/** Chords the browser keeps for itself, in the order the recorder reports them (Control, Alt, Shift, Meta). */
const PRIMARY_KEYS = ["T", "N", "W", "F", "R", "L", "D", "H", "J", "O", "P", "S", "U", "G"];
const SHIFTED_KEYS = ["T", "N", "W", "I", "J"];
const RESERVED = isMac
  ? [...PRIMARY_KEYS.map((k) => `Meta+${k}`), ...SHIFTED_KEYS.map((k) => `Shift+Meta+${k}`), "Meta+Q"]
  : [...PRIMARY_KEYS.map((k) => `Control+${k}`), ...SHIFTED_KEYS.map((k) => `Control+Shift+${k}`)];
const RESERVED_LIST = RESERVED.join(",");

const SPOKEN_KEY: Record<string, string> = {
  ArrowLeft: "Left arrow",
  ArrowRight: "Right arrow",
  ArrowUp: "Up arrow",
  ArrowDown: "Down arrow",
  Enter: "Return",
  " ": "Space",
};

function cx(...parts: (string | false | null | undefined)[]): string {
  return parts.filter(Boolean).join(" ");
}

function partName(name: string): string {
  return `app.settingsshortcuts.${name}`;
}

/** Whitespace-separated words of the query, lower-cased. Empty means no search. */
function queryTokens(query: string): string[] {
  return query.trim().toLowerCase().split(/\s+/).filter((token) => token !== "");
}

/** A row matches when every word is in its group, title, keywords or combo text (case-insensitive). */
function rowMatches(item: MainRow, tokens: string[], accel: string): boolean {
  const haystack = `${GROUP} ${item.label} shortcut hotkey key binding ${accel}`.toLowerCase();
  return tokens.every((token) => haystack.includes(token));
}

/** Splits "Mod+Alt+K" into its modifiers and key. A key of "+" is kept whole. */
function splitAccel(accel: string): { mods: string[]; key: string } {
  const key = accel.length > 1 && accel.endsWith("+") ? "+" : accel.slice(accel.lastIndexOf("+") + 1);
  const mods = accel
    .slice(0, accel.length - key.length)
    .split("+")
    .filter((name) => name !== "");
  return { mods, key };
}

/** A modifier in the stored form. Mod is the platform's primary key (Command on macOS, Ctrl elsewhere). */
function storedName(name: string): string {
  if (name === "Alt" || name === "Option") return "Alt";
  if (name === "Shift") return "Shift";
  if (name === "Meta" || name === "Super") return isMac ? "Mod" : "Meta";
  if (name === "Control" || name === "Ctrl") return isMac ? "Ctrl" : "Mod";
  return "Mod";
}

/** The stored form of a combo: the primary modifier, then Alt, then Shift, then the key. */
function canonical(combo: string): string {
  const { mods, key } = splitAccel(combo);
  const rank: Record<string, number> = { Alt: 1, Shift: 2 };
  const names = mods.map(storedName).sort((a, b) => (rank[a] ?? 0) - (rank[b] ?? 0));
  return [...names, key].join("+");
}

/** The stored combo in the names the Keycap widget draws (Ctrl or Command, Alt, Shift, key). */
function keycapAccel(accel: string): string {
  if (accel === "") return "";
  const { mods, key } = splitAccel(accel);
  const names = mods.map((name) => {
    if (name === "Mod") return isMac ? "Meta" : "Ctrl";
    if (name === "Ctrl") return isMac ? "Control" : "Ctrl";
    return name;
  });
  return [...names, key].join("+");
}

/** The spoken name of a stored combo: "Ctrl Shift Z" or "Command Shift Z", "Alt Shift Left arrow". */
function speak(accel: string): string {
  const { mods, key } = splitAccel(accel);
  const names = mods.map((name) => {
    switch (storedName(name)) {
      case "Alt":
        return isMac ? "Option" : "Alt";
      case "Shift":
        return "Shift";
      case "Ctrl":
        return isMac ? "Control" : "Ctrl";
      case "Meta":
        return "Super";
      default:
        return isMac ? "Command" : "Ctrl";
    }
  });
  return [...names, SPOKEN_KEY[key] ?? key].join(" ");
}

/** The accel of a command in the list, or its default while the list has not loaded. */
function accelOf(list: Binding[] | undefined, command: string): string {
  const found = list?.find((item) => item.command === command);
  return found ? found.accel : (DEFAULTS[command] ?? "");
}

function withoutKey(map: Record<string, string>, key: string): Record<string, string> {
  const next = { ...map };
  delete next[key];
  return next;
}

/** Fades the pane in on each mount; a new pane (a section switch) mounts afresh at the top. */
function PaneScroll({ children }: { children?: React.ReactNode }) {
  const [shown, setShown] = React.useState(false);
  React.useEffect(() => {
    const frame = window.requestAnimationFrame(() => setShown(true));
    return () => window.cancelAnimationFrame(frame);
  }, []);
  return (
    <div
      role="tabpanel"
      data-part={partName("pane")}
      className={cx(
        "min-h-0 min-w-0 flex-1 overflow-y-auto px-[1.286rem] pt-[0.857rem] pb-[1.286rem]",
        "transition-opacity duration-(--duration-fast) ease-standard",
        shown ? "opacity-100" : "opacity-0",
      )}
    >
      {children}
    </div>
  );
}

interface ShortcutKeyProps {
  partKey: string;
  accel: string;
  recording: boolean;
  onStart: () => void;
  onCapture: (combo: string | null) => void;
  onCancel: () => void;
  onRefused: (combo: string) => void;
}

/**
 * The keycap of one row. The Keycap widget names the key with its raw accelerator; the spoken name
 * (Left arrow, Ctrl, Command) is set on its button here, and the part names the button.
 */
function ShortcutKey({ partKey, accel, recording, onStart, onCapture, onCancel, onRefused }: ShortcutKeyProps) {
  const ref = React.useRef<HTMLDivElement | null>(null);
  useParts(ref, { keycap: partName(partKey) });
  const spoken = accel === "" ? "None" : speak(accel);
  React.useLayoutEffect(() => {
    ref.current?.querySelector("button[data-slot='keycap']")?.setAttribute("aria-label", spoken);
  });
  return (
    <Keycap
      ref={ref}
      mode={recording ? "recording" : "idle"}
      binding="default"
      accel={keycapAccel(accel)}
      reserved={RESERVED_LIST}
      onStartRecording={onStart}
      onCapture={onCapture}
      onCancel={onCancel}
      onRefused={onRefused}
    />
  );
}

interface ShortcutLineProps {
  item: MainRow;
  accel: string;
  recording: boolean;
  error: string;
  onStart: () => void;
  onCapture: (combo: string | null) => void;
  onCancel: () => void;
  onRefused: (combo: string) => void;
  onReset: () => void;
}

/** One row: the title on the left, the keycap on the right, a reset while it differs from its default. */
function ShortcutLine({ item, accel, recording, error, onStart, onCapture, onCancel, onRefused, onReset }: ShortcutLineProps) {
  return (
    <div data-part={partName(item.row ?? "row")} className="flex min-w-0 flex-col">
      <div className="group/row flex h-[2.5rem] min-w-0 items-center gap-2 pl-[0.571rem] pr-[0.714rem]">
        <Text as="div" size="md" tone="primary" overflow="truncate" className="min-w-0 flex-1 leading-[1.393rem]">
          {item.label}
        </Text>
        <ShortcutKey
          partKey={item.key ?? "keycap"}
          accel={accel}
          recording={recording}
          onStart={onStart}
          onCapture={onCapture}
          onCancel={onCancel}
          onRefused={onRefused}
        />
        {!recording && accel !== item.accel && (
          <Button
            data-part={partName("reset")}
            variant="ghost"
            size="sm"
            tone="muted"
            shape="icon"
            icon="restart_alt"
            iconSize={15}
            ariaLabel={`Reset ${item.label} to default`}
            description="Reset to default"
            reveal="row"
            onClick={onReset}
            className="shrink-0"
          />
        )}
      </div>
      {error !== "" && (
        <Text
          as="p"
          role="status"
          data-part={partName("shortcutError")}
          size="xs"
          tone="danger"
          className="pb-1 pl-[0.571rem]"
        >
          {error}
        </Text>
      )}
    </div>
  );
}

/**
 * embedded: the page inside the app's Settings dialog, which already draws the window, the search
 * and the section list; only the shortcuts pane is drawn. Standalone (its own contract) it draws all.
 */
export function ShortcutsPage({ onBindingChanged, embedded = false }: { onBindingChanged?: (label: string, accel: string) => void; embedded?: boolean }) {
  const qc = useQueryClient();
  const shortcuts = useShortcuts();
  const bindings = shortcuts.data;
  const write = useWrite(async (command: string, accel: string | null) => {
    const list = await ep.settings.bind(command, accel);
    qc.setQueryData(keys.shortcuts, list);
    return list;
  });

  const [open, setOpen] = React.useState(true);
  const [pane, setPane] = React.useState<PaneId>("shortcuts");
  const [query, setQuery] = React.useState("");
  const [scoped, setScoped] = React.useState(false);
  const [recording, setRecording] = React.useState<string | null>(null);
  const [errors, setErrors] = React.useState<Record<string, string>>({});
  // The bindings as the window opened (the first load), for the undo controls.
  const [baseline, setBaseline] = React.useState<Record<string, string> | null>(null);
  const inputRef = React.useRef<HTMLInputElement | null>(null);
  const queue = React.useRef<Promise<unknown>>(Promise.resolve());

  React.useEffect(() => {
    if (baseline === null && bindings) {
      setBaseline(Object.fromEntries(bindings.map((item) => [item.command, item.accel])));
    }
  }, [baseline, bindings]);

  const current = (command: string) => accelOf(bindings, command);
  const opened = (command: string) => baseline?.[command] ?? DEFAULTS[command] ?? "";
  const changedRows = MAIN_ROWS.filter((item) => current(item.command) !== opened(item.command));
  const anyChange = changedRows.length > 0;

  // Writes run one after another, so a later response never overtakes an earlier one.
  function run<T>(task: () => Promise<T>): Promise<T> {
    const next = queue.current.then(task);
    queue.current = next.catch(() => undefined);
    return next;
  }

  /** Saves one binding (null resets it to its default) and reports it. Called inside run(). */
  async function saveOne(command: string, label: string, accel: string | null) {
    const previous = qc.getQueryData<Binding[]>(keys.shortcuts);
    qc.setQueryData<Binding[]>(keys.shortcuts, (old) =>
      old?.map((item) => (item.command === command ? { ...item, accel: accel ?? DEFAULTS[command] ?? "" } : item)),
    );
    try {
      const saved = await write.mutateAsync([command, accel]);
      setErrors((prev) => withoutKey(prev, command));
      onBindingChanged?.(label, accelOf(saved, command));
    } catch (error) {
      qc.setQueryData(keys.shortcuts, previous);
      const message = error instanceof Error ? error.message : "Could not save this shortcut.";
      setErrors((prev) => ({ ...prev, [command]: message }));
    }
  }

  function commit(command: string, label: string, accel: string | null) {
    void run(() => saveOne(command, label, accel));
  }

  /** Puts each row back to the accel it had when the window opened. */
  function restore(items: MainRow[]) {
    setErrors({});
    void run(async () => {
      for (const item of items) await saveOne(item.command, item.label, opened(item.command));
    });
  }

  const tokens = queryTokens(query);
  const searching = tokens.length > 0;
  const rows = MAIN_ROWS.filter((item) => !searching || rowMatches(item, tokens, current(item.command)));
  // General and About hold no rows on this screen, so they never match a query.
  const counts: Record<PaneId, number> = { shortcuts: searching ? rows.length : 0, general: 0, about: 0 };
  const anyMatch = counts.shortcuts > 0;

  function editQuery(next: string) {
    setQuery(next);
    setScoped(false);
  }

  function clearQuery() {
    setQuery("");
    setScoped(false);
  }

  /** A click on a section with matches narrows the search to it; one without matches clears the search. */
  function choosePane(next: PaneId) {
    setPane(next);
    if (!searching) return;
    if (counts[next] > 0) {
      setScoped(true);
    } else {
      clearQuery();
    }
  }

  function focusWell(event: React.MouseEvent<HTMLDivElement>) {
    if ((event.target as Element).closest("button, input")) return;
    event.preventDefault();
    inputRef.current?.focus();
  }

  /** Escape clears a query first; with no query it closes Settings. Not while a shortcut is recorded. */
  function onDialogKeyDown(event: React.KeyboardEvent<HTMLDivElement>) {
    if (event.key !== "Escape" || recording !== null) return;
    event.preventDefault();
    if (query !== "") {
      clearQuery();
      return;
    }
    setOpen(false);
  }

  function discardAll() {
    restore(changedRows);
  }

  const heading = searching && !scoped ? `Shortcuts · ${GROUP}` : GROUP;

  let body: React.ReactNode = null;
  if (searching && !anyMatch) {
    body = (
      <EmptyState
        scope="list"
        role="status"
        data-part={partName("noMatch")}
        className="pt-[4.5rem]"
        title="No settings match"
        subtitle={`Nothing here for “${query.trim()}”.`}
        icon="search"
        iconSize={30}
      />
    );
  } else if (pane === "shortcuts" || searching) {
    body = (
      <div className="flex min-w-0 flex-col gap-[1.214rem]">
        {!searching && (
          <Text data-part={partName("intro")} as="p" size="sm" tone="muted">
            {INTRO}
          </Text>
        )}
        <section className="flex min-w-0 flex-col gap-[0.5rem]">
          <div className="flex min-w-0 items-center gap-2">
            <Text
              data-part={partName("groupMain")}
              as="div"
              role="heading"
              aria-level={2}
              size="xs"
              weight="semibold"
              tone="muted"
              // Caps by font-variant, not text-transform: the contract reads the group text as written.
              className="shrink-0 tracking-[0.12em] [font-variant-caps:all-small-caps]"
            >
              {heading}
            </Text>
            <span aria-hidden="true" className="block h-px min-w-4 flex-1 bg-edge-2" />
            {anyChange && (
              <Button
                data-part={partName("discardGroup")}
                variant="ghost"
                size="sm"
                tone="muted"
                shape="icon"
                icon="undo"
                iconSize={14}
                ariaLabel="Discard changes in this group"
                description="Discard changes made since this window opened"
                onClick={discardAll}
                className="shrink-0"
              />
            )}
          </div>
          <div className="divide-y divide-edge-1 overflow-hidden rounded-(--radius-panel) border border-edge-2 bg-surface-2">
            {rows.map((item) => (
              <ShortcutLine
                key={item.command}
                item={item}
                accel={current(item.command)}
                recording={recording === item.command}
                error={errors[item.command] ?? ""}
                onStart={() => {
                  setErrors((prev) => withoutKey(prev, item.command));
                  setRecording(item.command);
                }}
                onCapture={(combo) => {
                  setRecording(null);
                  commit(item.command, item.label, combo === null ? "" : canonical(combo));
                }}
                onCancel={() => setRecording(null)}
                onRefused={(combo) =>
                  setErrors((prev) => ({ ...prev, [item.command]: `${speak(canonical(combo))} is reserved by the browser.` }))
                }
                onReset={() => commit(item.command, item.label, null)}
              />
            ))}
          </div>
        </section>
      </div>
    );
  }

  if (embedded) return <PaneScroll key="shortcuts">{body}</PaneScroll>;

  return (
    <Overlay
      open={open}
      onOpenChange={(next) => {
        if (!next) setOpen(false);
      }}
      captureEscape={false}
      aria-label="Settings"
      data-part={partName("dialog")}
      onKeyDown={onDialogKeyDown}
      className="flex h-[min(37.143rem,100%)] max-h-full w-[min(45.714rem,100%)] flex-col overflow-hidden"
    >
      <header className="flex h-[3.286rem] shrink-0 items-center gap-2 border-b border-edge-1 bg-surface-1 pl-[1.286rem] pr-[0.571rem]">
        <Heading as="h1" size="sm" data-part={partName("title")}>
          Settings
        </Heading>
        <div className="ml-auto flex shrink-0 items-center gap-1">
          {anyChange && (
            <Button
              data-part={partName("discardAll")}
              variant="ghost"
              size="xl"
              shape="icon"
              tone="strong"
              icon="undo"
              iconSize={19}
              ariaLabel="Discard changes"
              description="Discard changes made since this window opened"
              onClick={discardAll}
            />
          )}
          <Button
            data-part={partName("close")}
            variant="ghost"
            size="sm"
            shape="icon"
            tone="muted"
            icon="close"
            iconSize={16}
            ariaLabel="Close settings"
            onClick={() => setOpen(false)}
          />
        </div>
      </header>
      <div className="flex min-h-0 flex-1">
        <nav className="flex w-[10.929rem] shrink-0 flex-col gap-[0.571rem] border-r border-edge-1 bg-surface-1 px-[0.5rem] pt-[0.5rem]">
          <div
            onMouseDown={focusWell}
            className="flex h-[1.786rem] min-w-0 items-center gap-1.5 rounded-(--radius-field) bg-wash-2 pl-[0.643rem] pr-0.5 focus-within:ring-2 focus-within:ring-focus-1"
          >
            <Icon name="search" size={14} tone="muted" />
            <Input
              ref={inputRef}
              data-part={partName("search")}
              ariaLabel="Search settings"
              placeholder="Search"
              size="sm"
              mode="bare"
              spellCheck={false}
              value={query}
              onValueChange={editQuery}
              className="min-w-0 flex-1"
            />
            {query !== "" && (
              <Button
                data-part={partName("searchClear")}
                variant="ghost"
                size="sm"
                tone="muted"
                shape="icon"
                icon="close"
                iconSize={14}
                ariaLabel="Clear search"
                description="Clear search"
                retainFocus
                onClick={() => {
                  clearQuery();
                  inputRef.current?.focus();
                }}
                className="shrink-0"
              />
            )}
          </div>
          <TabList
            label="Settings sections"
            orientation="vertical"
            className="gap-[0.143rem] p-0"
            value={pane}
            onValueChange={(next) => choosePane(next as PaneId)}
          >
            {PANES.map((item) => {
              const dimmed = searching && counts[item.id] === 0;
              const accent = !searching || scoped;
              return (
                <TabListItem
                  key={item.id}
                  value={item.id}
                  onClick={() => choosePane(item.id)}
                  data-part={partName(item.part)}
                  className={cx(
                    "h-[2.143rem] gap-2 px-[0.643rem] font-normal",
                    accent ? "data-[state=active]:bg-accent-500/10" : "data-[state=active]:bg-wash-2",
                    dimmed && "opacity-40",
                  )}
                >
                  <span className="flex w-full min-w-0 items-center gap-2">
                    <Icon name={item.icon} size={16} tone="muted" />
                    <Text size="md" overflow="truncate" className="min-w-0 flex-1">
                      {item.label}
                    </Text>
                    {item.id === "shortcuts" && (
                      <Badge
                        data-part={partName("shortcutsMatches")}
                        role="status"
                        tone="recessed"
                        visible={searching && counts.shortcuts > 0}
                      >
                        {counts.shortcuts}
                      </Badge>
                    )}
                  </span>
                </TabListItem>
              );
            })}
          </TabList>
        </nav>
        <PaneScroll key={pane}>{body}</PaneScroll>
      </div>
    </Overlay>
  );
}
