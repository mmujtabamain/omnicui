// The Settings page (web form of the Settings window): a plain heading, a section list with search,
// and the pane content. Settings are stored per account; each change is saved as it is made, and
// the discard controls write the values from when the page opened back.
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
  const write = useWrite(ep.settings.update);

  // The open-time values, taken once when the page first has them. `last` keeps the latest values
  // seen, so the page holds still if the settings query is refetched into a new cache.
  const opened = React.useRef<{ baseline: Settings; last: Settings } | null>(null);
  if (data !== undefined) {
    if (opened.current === null) opened.current = { baseline: data, last: data };
    opened.current.last = data;
  }

  const [pane, setPane] = React.useState<PaneId>("general");
  const [query, setQuery] = React.useState("");
  const [scoped, setScoped] = React.useState(false);
  const [failed, setFailed] = React.useState<Partial<Record<PrefField, boolean>>>({});

  const values: Settings | undefined = data ?? opened.current?.last;
  const baseline: Settings | undefined = opened.current?.baseline;
  if (values === undefined || baseline === undefined) return null;

  const tokens = queryTokens(query);
  const searching = tokens.length > 0;
  const counts: Record<PaneId, number> = {
    general: searching ? GENERAL_ROWS.filter((row) => rowMatches(row, tokens)).length : 0,
    shortcuts: 0,
    about: 0,
  };
  const anyMatch = counts.general + counts.shortcuts + counts.about > 0;
  const anyChange = PREF_FIELDS.some((field) => values[field] !== baseline[field]);

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

  function discard(fields: PrefField[]) {
    const patch: Partial<Settings> = {};
    for (const field of fields) Object.assign(patch, { [field]: baseline![field] });
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
        baseline={baseline}
        tokens={tokens}
        scoped={scoped}
        failed={failed}
        onChange={commit}
        onDiscard={discard}
      />
    );
  } else {
    body = pane === "shortcuts" ? <ShortcutsPage /> : <AboutPage />;
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
                  data-part={PANE_PART[item.id]}
                  className={cx(
                    "h-[2.143rem] gap-2 font-normal",
                    accent && "data-[state=active]:bg-accent-500/10",
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
