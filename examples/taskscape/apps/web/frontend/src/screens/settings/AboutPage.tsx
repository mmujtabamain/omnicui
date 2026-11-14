// The Settings screen in the web build (the 640 x 520 dialog, web.md): a page heading, a section list with
// search, and the About pane. General and Shortcuts are the other panes of the same screen. Settings are kept
// per account and saved as each change is made. The discard controls put back the values from when this
// screen opened (F10-033), so a reset can be undone while the screen is open.
import * as React from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Badge } from "@taskscape/ui/badge";
import { Button } from "@taskscape/ui/button";
import { Heading } from "@taskscape/ui/heading";
import { Icon } from "@taskscape/ui/icon";
import { Input } from "@taskscape/ui/input";
import { TabList, TabListItem } from "@taskscape/ui/tab-list";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { keys, useSettings, useShortcuts, useWrite } from "../../api/queries";
import type { Binding, Settings } from "../../api/types";
import { GENERAL_ROWS, GeneralPage } from "./GeneralPage";
import { PANES, cx, queryTokens, rowMatches } from "./shared";
import type { PaneId, PrefField, SearchRow } from "./shared";

const PREF_FIELDS: PrefField[] = ["theme", "reduced_motion", "show_completed"];

/** What Restore defaults puts back (appearance and task preferences; shortcuts are reset separately). */
const DEFAULT_PREFS = { theme: "system", reduced_motion: "system", show_completed: true } as const;

const PANE_PART: Record<PaneId, string> = {
  general: "app.settingsabout.navGeneral",
  shortcuts: "app.settingsabout.navShortcuts",
  about: "app.settingsabout.navAbout",
};

/** Owner copy is still to be written; this placeholder keeps the row present. */
const APP_DESCRIPTION = "A task manager for lists, projects and subtasks.";
const RESTORE_DESCRIPTION =
  "Puts appearance and task preferences — and every keyboard shortcut — back the way they shipped. Your tasks are untouched.";

/** The rows of this pane. Their text (title, description, keywords) is what search matches. */
const ABOUT_ROWS: SearchRow[] = [
  {
    id: "application",
    pane: "about",
    group: "application",
    title: "Taskscape",
    description: APP_DESCRIPTION,
    keywords: "about version application name",
  },
  {
    id: "restore",
    pane: "about",
    group: "reset",
    title: "Restore defaults",
    description: RESTORE_DESCRIPTION,
    keywords: "reset clear defaults factory revert erase preferences",
  },
];

/** The values from when this screen opened. */
interface Baseline {
  settings: Settings;
  shortcuts: Binding[];
}

export interface AboutPageProps {
  /** ctx.version. "" means no version is set, and the row shows an em dash. */
  version?: string;
  /** Runs when the confirmed reset runs. */
  onResetAll?: () => void;
  /** Inside the app's Settings dialog, which draws the window, search and sections: only the pane. */
  embedded?: boolean;
}

export function AboutPage({ version = "", onResetAll, embedded = false }: AboutPageProps) {
  const qc = useQueryClient();
  const settingsQuery = useSettings();
  const shortcutsQuery = useShortcuts();
  const settings = settingsQuery.data;
  const bindings = shortcutsQuery.data ?? [];

  // The open-time values are taken once, when both lists have loaded.
  const [baseline, setBaseline] = React.useState<Baseline | null>(null);
  const shortcutsSettled = shortcutsQuery.isSuccess || shortcutsQuery.isError;
  if (baseline === null && settings !== undefined && shortcutsSettled) {
    setBaseline({ settings, shortcuts: shortcutsQuery.data ?? [] });
  }

  const [pane, setPane] = React.useState<PaneId>("about");
  const [query, setQuery] = React.useState("");
  const [scoped, setScoped] = React.useState(false);
  const [confirming, setConfirming] = React.useState(false);
  const [failed, setFailed] = React.useState<Partial<Record<PrefField, boolean>>>({});
  const searchRef = React.useRef<HTMLInputElement | null>(null);
  const cancelRef = React.useRef<HTMLButtonElement | null>(null);
  const resetRef = React.useRef<HTMLButtonElement | null>(null);
  // Set when the confirmation closes, so focus goes back to the single reset button.
  const leaving = React.useRef(false);

  /** Saves preference changes and shortcut restores; the server's reply replaces the cached values. */
  const save = useWrite(async (patch: Partial<Settings>, binds: Binding[]) => {
    if (Object.keys(patch).length > 0) {
      const saved = await ep.settings.update(patch);
      qc.setQueryData<Settings>(keys.settings, saved);
    }
    for (const binding of binds) {
      const list = await ep.settings.bind(binding.command, binding.accel);
      qc.setQueryData<Binding[]>(keys.shortcuts, list);
    }
  });

  /** Restore defaults: every preference and every shortcut. One failed shortcut does not stop the rest. */
  const reset = useWrite(async () => {
    const saved = await ep.settings.update({ ...DEFAULT_PREFS });
    qc.setQueryData<Settings>(keys.settings, saved);
    const current = qc.getQueryData<Binding[]>(keys.shortcuts) ?? [];
    for (const binding of current) {
      try {
        const list = await ep.settings.bind(binding.command, null);
        qc.setQueryData<Binding[]>(keys.shortcuts, list);
      } catch {
        // Ignored on purpose: the other shortcuts are still reset (F10-072).
      }
    }
    await qc.invalidateQueries({ queryKey: keys.shortcuts });
  });

  React.useEffect(() => {
    if (confirming) {
      cancelRef.current?.focus();
      return;
    }
    if (leaving.current) {
      leaving.current = false;
      resetRef.current?.focus();
    }
  }, [confirming]);

  if (settings === undefined || baseline === null) return null;

  const values: Settings = settings;
  const base: Baseline = baseline;
  const changedField = (field: PrefField) => values[field] !== base.settings[field];
  const changedPrefs = PREF_FIELDS.filter((field) => changedField(field));
  const restoreBindings: Binding[] = bindings.flatMap((binding) => {
    const open = base.shortcuts.find((item) => item.command === binding.command);
    return open !== undefined && open.accel !== binding.accel ? [open] : [];
  });
  const anyChange = changedPrefs.length > 0 || restoreBindings.length > 0;

  const tokens = queryTokens(query);
  const searching = tokens.length > 0;
  const aboutCount = searching ? ABOUT_ROWS.filter((row) => rowMatches(row, tokens)).length : 0;
  const generalCount = searching ? GENERAL_ROWS.filter((row) => rowMatches(row, tokens)).length : 0;
  const counts: Record<PaneId, number> = { general: generalCount, shortcuts: 0, about: aboutCount };
  const paneEmpty = searching && counts[pane] === 0;
  const rowShown = (id: string) => !searching || ABOUT_ROWS.some((row) => row.id === id && rowMatches(row, tokens));
  const heading = (name: string) => (searching && !scoped ? `About · ${name}` : name);

  /** Shows the change at once, saves it, and puts the fields back if the save fails. */
  const commit = (patch: Partial<Settings>, binds: Binding[] = []) => {
    const fields = PREF_FIELDS.filter((field) => field in patch && patch[field] !== values[field]);
    if (fields.length === 0 && binds.length === 0) return;
    const next: Partial<Settings> = {};
    const before: Partial<Settings> = {};
    for (const field of fields) {
      Object.assign(next, { [field]: patch[field] });
      Object.assign(before, { [field]: values[field] });
    }
    qc.setQueryData<Settings>(keys.settings, (old) => (old ? ({ ...old, ...next } as Settings) : old));
    setFailed((prev) => {
      const copy = { ...prev };
      for (const field of fields) copy[field] = false;
      return copy;
    });
    save.mutateAsync([next, binds]).catch(() => {
      qc.setQueryData<Settings>(keys.settings, (old) => (old ? ({ ...old, ...before } as Settings) : old));
      setFailed((prev) => {
        const copy = { ...prev };
        for (const field of fields) copy[field] = true;
        return copy;
      });
    });
  };

  /** Writes back the open-time value of each field and shortcut given. */
  const discard = (fields: PrefField[], binds: Binding[]) => {
    const patch: Partial<Settings> = {};
    for (const field of fields) Object.assign(patch, { [field]: base.settings[field] });
    commit(patch, binds);
  };

  const editQuery = (next: string) => {
    setQuery(next);
    setScoped(false);
  };

  const clearQuery = () => {
    setQuery("");
    setScoped(false);
    searchRef.current?.focus();
  };

  /** A click on a tab with matches narrows the search to it; one without matches clears the search. */
  const choosePane = (next: PaneId) => {
    setPane(next);
    if (!searching) return;
    if (counts[next] > 0) {
      setScoped(true);
    } else {
      setQuery("");
      setScoped(false);
    }
  };

  const runReset = () => {
    leaving.current = true;
    setConfirming(false);
    onResetAll?.();
    reset.mutate([]);
  };

  const cancelConfirm = () => {
    leaving.current = true;
    setConfirming(false);
  };

  const aboutBody = (
    <div className="flex min-w-0 flex-col gap-[1.286rem]">
      <Group label={heading("Application")} labelPart="app.settingsabout.applicationHeader" visible={rowShown("application")}>
        <div className="flex min-w-0 items-center gap-3 px-[0.571rem] py-[0.5rem]">
          <div className="flex min-w-0 flex-1 flex-col gap-[0.143rem]">
            <Text data-part="app.settingsabout.appName" as="div" size="md" tone="primary" className="leading-[1.393rem]">
              Taskscape
            </Text>
            <Text
              data-part="app.settingsabout.appDescription"
              as="p"
              size="xs"
              tone="muted"
              className="text-[0.821rem] leading-[1.232rem]"
            >
              {APP_DESCRIPTION}
            </Text>
          </div>
          <Text
            data-part="app.settingsabout.version"
            as="span"
            size="xs"
            font="mono"
            tone="muted"
            numeric="tabular"
            className="shrink-0 text-[0.821rem]"
          >
            {version === "" ? "—" : version}
          </Text>
        </div>
      </Group>

      <Group label={heading("Reset")} labelPart="app.settingsabout.resetHeader" visible={rowShown("restore")}>
        <div className="flex min-w-0 flex-col gap-[0.5rem] px-[0.571rem] py-[0.5rem]">
          <div className="flex min-w-0 flex-col gap-[0.143rem]">
            <Text data-part="app.settingsabout.restoreDefaults" as="div" size="md" tone="primary" className="leading-[1.393rem]">
              Restore defaults
            </Text>
            <Text
              data-part="app.settingsabout.restoreDescription"
              as="p"
              size="xs"
              tone="muted"
              className="text-[0.821rem] leading-[1.232rem]"
            >
              {RESTORE_DESCRIPTION}
            </Text>
          </div>
          {confirming ? (
            <div className="flex min-w-0 gap-2">
              <Button
                ref={cancelRef}
                data-part="app.settingsabout.cancelReset"
                variant="outline"
                tone="strong"
                size="lg"
                width="full"
                label="Cancel"
                onClick={cancelConfirm}
                onKeyDown={(event) => {
                  if (event.key === "Escape") {
                    event.preventDefault();
                    cancelConfirm();
                  }
                }}
                className="text-[0.893rem] font-semibold"
              />
              <Button
                data-part="app.settingsabout.confirmReset"
                variant="danger"
                size="lg"
                width="full"
                label="Reset all settings"
                ariaLabel="Reset all settings"
                onClick={runReset}
                onKeyDown={(event) => {
                  // Enter arms but never confirms (decision 10). Space or a click confirms.
                  if (event.key === "Enter") event.preventDefault();
                  if (event.key === "Escape") {
                    event.preventDefault();
                    cancelConfirm();
                  }
                }}
                className="text-[0.893rem] font-semibold"
              />
            </div>
          ) : (
            <Button
              ref={resetRef}
              data-part="app.settingsabout.resetButton"
              variant="outline"
              tone="strong"
              size="lg"
              width="full"
              label="Reset all settings…"
              ariaLabel="Reset all settings…"
              onClick={() => setConfirming(true)}
              className="text-[0.893rem] font-semibold"
            />
          )}
          {reset.isError && (
            <Text as="p" role="status" size="xs" tone="danger">
              Could not reset settings. Try again.
            </Text>
          )}
        </div>
      </Group>
    </div>
  );

  if (embedded) return <PaneFrame key="about" pane="about">{aboutBody}</PaneFrame>;

  return (
    <div
      data-part="app.settingsabout.window"
      role="dialog"
      aria-label="Settings"
      className="flex h-dvh min-w-0 flex-col overflow-hidden bg-surface-2 text-content-1"
    >
      <header className="flex h-[3.286rem] shrink-0 items-center gap-2 border-b border-edge-1 bg-surface-1 px-[1.286rem]">
        <Heading as="h1" size="sm" data-part="app.settingsabout.title">
          Settings
        </Heading>
        <Button
          data-part="app.settingsabout.discardAll"
          variant="ghost"
          size="xl"
          shape="icon"
          tone="strong"
          icon="undo"
          iconSize={19}
          ariaLabel="Discard all changes"
          description="Discard all settings changes since this page opened"
          onClick={() => discard(changedPrefs, restoreBindings)}
          className={cx("ml-auto", !anyChange && "hidden")}
        />
      </header>
      <div className="flex min-h-0 flex-1">
        <nav className="flex w-44 shrink-0 flex-col gap-2 border-r border-edge-1 bg-surface-1 px-2 pt-2">
          <div
            className="flex h-[1.75rem] min-w-0 items-center gap-1.5 rounded-(--radius-field) bg-wash-2 pl-[0.643rem] pr-0.5 focus-within:ring-2 focus-within:ring-focus-1"
            onMouseDown={(event) => {
              if ((event.target as Element).closest("button, input")) return;
              event.preventDefault();
              searchRef.current?.focus();
            }}
          >
            <Icon name="search" size={14} tone="muted" />
            <Input
              ref={searchRef}
              data-part="app.settingsabout.search"
              type="search"
              ariaLabel="Search settings"
              placeholder="Search"
              size="sm"
              mode="bare"
              spellCheck={false}
              value={query}
              onValueChange={editQuery}
              onCancel={() => {
                if (query !== "") clearQuery();
              }}
              className="min-w-0 flex-1 [&::-webkit-search-cancel-button]:appearance-none"
            />
            <Button
              data-part="app.settingsabout.searchClear"
              variant="ghost"
              size="sm"
              tone="muted"
              shape="icon"
              icon="close"
              iconSize={14}
              ariaLabel="Clear search"
              description="Clear search"
              retainFocus
              onClick={clearQuery}
              className={cx("shrink-0", !searching && "hidden")}
            />
          </div>
          <TabList
            data-part="app.settingsabout.sectionNav"
            label="Settings sections"
            orientation="vertical"
            className="gap-[0.143rem] p-0"
            value={pane}
            onValueChange={(next) => choosePane(next as PaneId)}
          >
            {PANES.map((item) => {
              const dimmed = searching && counts[item.id] === 0;
              return (
                <TabListItem
                  key={item.id}
                  value={item.id}
                  aria-controls={`settings-panel-${item.id}`}
                  // A click on the tab that is already open (About while searching) still scopes the search.
                  onClick={() => choosePane(item.id)}
                  data-part={PANE_PART[item.id]}
                  className={cx("h-[2.143rem] gap-2 px-[0.643rem] font-normal", dimmed && "opacity-40")}
                >
                  <span className="flex w-full min-w-0 items-center gap-2">
                    <Icon name={item.icon} size={16} tone="muted" />
                    <Text size="md" overflow="truncate" className="min-w-0 flex-1">
                      {item.label}
                    </Text>
                    {item.id === "about" && (
                      <Badge
                        data-part="app.settingsabout.aboutMatches"
                        tone="recessed"
                        visible={searching && aboutCount > 0}
                      >
                        {aboutCount}
                      </Badge>
                    )}
                  </span>
                </TabListItem>
              );
            })}
          </TabList>
        </nav>
        <PaneFrame key={pane} pane={pane}>
          <div
            data-part="app.settingsabout.emptyState"
            className={cx("flex min-w-0 flex-col items-center pt-[4.5rem] text-center", !paneEmpty && "hidden")}
          >
            <Icon name="search" size={30} tone="muted" />
            <Text as="div" size="md" weight="medium" tone="secondary" className="mt-2 text-[1.071rem]">
              No settings match
            </Text>
            <Text as="div" size="caption" tone="muted" className="mt-1">
              {`Nothing here for “${query.trim()}”.`}
            </Text>
          </div>
          <div className={cx(paneEmpty && "hidden")}>
            {pane === "about" && aboutBody}
            {pane === "general" && (
              <GeneralPage
                values={values}
                changed={changedField}
                tokens={tokens}
                scoped={scoped}
                failed={failed}
                onChange={(patch) => commit(patch)}
                onDiscard={(fields) => discard(fields, [])}
              />
            )}
          </div>
        </PaneFrame>
      </div>
    </div>
  );
}

/** An eyebrow label over a card of rows. The label is a heading; no group discard (About has none). */
function Group({
  label,
  labelPart,
  visible,
  children,
}: {
  label: string;
  labelPart: string;
  visible: boolean;
  children: React.ReactNode;
}) {
  return (
    <section className={cx("flex min-w-0 flex-col gap-2", !visible && "hidden")}>
      <div className="flex min-w-0 items-center gap-2">
        <Text
          as="div"
          role="heading"
          aria-level={2}
          data-part={labelPart}
          size="xs"
          weight="semibold"
          tone="muted"
          case="uppercase"
          className="text-[0.75rem] tracking-[0.12em]"
        >
          {label}
        </Text>
        <span aria-hidden="true" className="block h-px min-w-4 flex-1 bg-edge-2" />
      </div>
      <div className="divide-y divide-edge-1 overflow-hidden rounded-(--radius-panel) border border-edge-2 bg-surface-2">
        {children}
      </div>
    </section>
  );
}

/** The scrolling content of the active pane. It fades in on each switch of pane. */
function PaneFrame({ pane, children }: { pane: PaneId; children: React.ReactNode }) {
  const [shown, setShown] = React.useState(false);
  React.useEffect(() => {
    const frame = window.requestAnimationFrame(() => setShown(true));
    return () => window.cancelAnimationFrame(frame);
  }, []);
  return (
    <div
      role="tabpanel"
      id={`settings-panel-${pane}`}
      data-part="app.settingsabout.panel"
      className={cx(
        "min-h-0 min-w-0 flex-1 overflow-y-auto px-[1.286rem] py-[0.857rem]",
        "transition-opacity duration-(--duration-fast) ease-standard",
        shown ? "opacity-100" : "opacity-0",
      )}
    >
      {children}
    </div>
  );
}
