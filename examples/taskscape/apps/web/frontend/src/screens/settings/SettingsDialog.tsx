// The Settings page (web form of the Settings window): a plain heading, a section list with search,
// and the pane content. Settings are stored per account and saved as each change is made. The
// discard controls write back each setting's value from before this page first changed it (the
// open-time value), read from the server just before that first write.
import * as React from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Badge } from "@taskscape/ui/badge";
import { Button } from "@taskscape/ui/button";
import { Heading } from "@taskscape/ui/heading";
import { Icon } from "@taskscape/ui/icon";
import { TabList, TabListItem } from "@taskscape/ui/tab-list";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { keys as queryKeys, useSettings, useWrite } from "../../api/queries";
import type { Settings } from "../../api/types";
import { AboutPage } from "./AboutPage";
import { GENERAL_ROWS, GeneralPage } from "./GeneralPage";
import { ShortcutsPage } from "./ShortcutsPage";
import {
  EmptyResults,
  PANES,
  PaneFrame,
  PrefField,
  PaneId,
  SearchBox,
  cx,
  queryTokens,
  rowMatches,
} from "./shared";

const PREF_FIELDS: PrefField[] = ["theme", "reduced_motion", "show_completed"];

const PANE_PART: Record<PaneId, string> = {
  general: "app.settingsgeneral.generalTab",
  shortcuts: "app.settingsgeneral.shortcutsTab",
  about: "app.settingsgeneral.aboutTab",
};

export function SettingsDialog() {
  const qc = useQueryClient();
  const { data } = useSettings();

  // The latest values seen, so the page holds still while the settings query is refetched.
  const lastSeen = React.useRef<Settings | null>(null);
  if (data !== undefined) lastSeen.current = data;
  const values: Settings | null = data ?? lastSeen.current;

  // Open-time values, per field: filled in just before this page's first write to the field.
  const [baseline, setBaseline] = React.useState<Partial<Settings>>({});
  const baselineReads = React.useRef<Partial<Record<PrefField, Promise<void>>>>({});

  function captureBaseline(fields: PrefField[]): Promise<void> {
    const waits = fields.map((field) => {
      if (!baselineReads.current[field]) {
        baselineReads.current[field] = ep.settings
          .get()
          .then((server) => {
            setBaseline((prev) => {
              if (field in prev) return prev;
              const next = { ...prev };
              Object.assign(next, { [field]: server[field] });
              return next;
            });
          })
          .catch((error: unknown) => {
            delete baselineReads.current[field];
            throw error;
          });
      }
      return baselineReads.current[field] as Promise<void>;
    });
    return Promise.all(waits).then(() => undefined);
  }

  const write = useWrite(async (patch: Partial<Settings>) => {
    await captureBaseline(Object.keys(patch) as PrefField[]);
    return ep.settings.update(patch);
  });

  const [pane, setPane] = React.useState<PaneId>("general");
  const [query, setQuery] = React.useState("");
  const [scoped, setScoped] = React.useState(false);
  const [failed, setFailed] = React.useState<Partial<Record<PrefField, boolean>>>({});

  if (values === null) return null;

  const changed = (field: PrefField) => field in baseline && values[field] !== baseline[field];

  const tokens = queryTokens(query);
  const searching = tokens.length > 0;
  const counts: Record<PaneId, number> = {
    general: searching ? GENERAL_ROWS.filter((row) => rowMatches(row, tokens)).length : 0,
    shortcuts: 0,
    about: 0,
  };
  const anyMatch = counts.general + counts.shortcuts + counts.about > 0;
  const anyChange = PREF_FIELDS.some((field) => changed(field));

  /** Shows the change at once, saves it, and puts the fields back if the save fails. */
  function commit(patch: Partial<Settings>) {
    const fields = PREF_FIELDS.filter((field) => field in patch && patch[field] !== values![field]);
    if (fields.length === 0) return;
    const next: Partial<Settings> = {};
    const before: Partial<Settings> = {};
    for (const field of fields) {
      Object.assign(next, { [field]: patch[field] });
      Object.assign(before, { [field]: values![field] });
    }
    qc.setQueryData<Settings>(queryKeys.settings, (old) => (old ? ({ ...old, ...next } as Settings) : old));
    setFailed((prev) => {
      const copy = { ...prev };
      for (const field of fields) copy[field] = false;
      return copy;
    });
    write.mutateAsync([next]).catch(() => {
      qc.setQueryData<Settings>(queryKeys.settings, (old) => (old ? ({ ...old, ...before } as Settings) : old));
      setFailed((prev) => {
        const copy = { ...prev };
        for (const field of fields) copy[field] = true;
        return copy;
      });
    });
  }

  /** Writes back the open-time value of each field that is known to have changed. */
  function discard(fields: PrefField[]) {
    const patch: Partial<Settings> = {};
    for (const field of fields) {
      if (field in baseline) Object.assign(patch, { [field]: baseline[field] });
    }
    commit(patch);
  }

  function editQuery(next: string) {
    setQuery(next);
    setScoped(false);
  }

  function clearQuery() {
    setQuery("");
    setScoped(false);
  }

  /** A click on a tab with matches narrows the search to it; one without matches clears the search. */
  function choosePane(next: PaneId) {
    setPane(next);
    if (!searching) return;
    if (counts[next] > 0) {
      setScoped(true);
    } else {
      setQuery("");
      setScoped(false);
    }
  }

  let body: React.ReactNode;
  if (searching && !anyMatch) {
    body = <EmptyResults query={query.trim()} />;
  } else if (searching || pane === "general") {
    body = (
      <GeneralPage
        values={values}
        changed={changed}
        tokens={tokens}
        scoped={scoped}
        failed={failed}
        onChange={commit}
        onDiscard={discard}
      />
    );
  } else {
    body = pane === "shortcuts" ? <ShortcutsPage embedded /> : <AboutPage embedded />;
  }

  return (
    <div className="flex h-dvh min-w-0 flex-col overflow-hidden bg-surface-2 text-content-1">
      <header className="flex h-[3.286rem] shrink-0 items-center gap-2 border-b border-edge-1 bg-surface-1 px-[1.286rem]">
        <Heading as="h1" size="sm" data-part="app.settingsgeneral.heading">
          Settings
        </Heading>
        <Button
          data-part="app.settingsgeneral.discardAll"
          variant="ghost"
          size="xl"
          shape="icon"
          tone="strong"
          icon="undo"
          iconSize={19}
          ariaLabel="Discard all changes"
          description="Discard all settings changes since this page opened"
          onClick={() => discard(PREF_FIELDS)}
          className={cx("ml-auto", !anyChange && "hidden")}
        />
      </header>
      <div className="flex min-h-0 flex-1">
        <nav className="flex w-44 shrink-0 flex-col gap-2 border-r border-edge-1 bg-surface-1 px-2 pt-2">
          <SearchBox value={query} onValueChange={editQuery} onClear={clearQuery} />
          <TabList
            data-part="app.settingsgeneral.sectionNav"
            label="Settings sections"
            orientation="vertical"
            className="gap-[0.143rem] p-0"
            value={pane}
            onValueChange={(next) => choosePane(next as PaneId)}
          >
            {PANES.map((item) => {
              const dimmed = searching && counts[item.id] === 0;
              const accent = !(searching && !scoped);
              return (
                <TabListItem
                  key={item.id}
                  value={item.id}
                  // A click on the tab that is already open (General while searching) still scopes the search.
                  onClick={() => choosePane(item.id)}
                  data-part={PANE_PART[item.id]}
                  className={cx(
                    "h-[2.143rem] gap-2 px-[0.643rem] font-normal",
                    accent ? "data-[state=active]:bg-accent-500/10" : "data-[state=active]:bg-transparent",
                    dimmed && "opacity-40",
                  )}
                >
                  <span className="flex w-full min-w-0 items-center gap-2">
                    <Icon name={item.icon} size={16} tone="muted" />
                    <Text size="md" overflow="truncate" className="min-w-0 flex-1">
                      {item.label}
                    </Text>
                    {item.id === "general" && (
                      <Badge
                        data-part="app.settingsgeneral.generalMatches"
                        tone="recessed"
                        visible={searching && counts.general > 0}
                      >
                        {counts.general}
                      </Badge>
                    )}
                  </span>
                </TabListItem>
              );
            })}
          </TabList>
        </nav>
        <PaneFrame key={searching ? "search" : pane}>{body}</PaneFrame>
      </div>
    </div>
  );
}
