// Shared parts of the Settings page: the search box, groups, rows, segmented choices and the
// empty result state. Shortcuts and About reuse them.
import * as React from "react";
import { Button } from "@taskscape/ui/button";
import { Icon } from "@taskscape/ui/icon";
import { Input } from "@taskscape/ui/input";
import { Text } from "@taskscape/ui/text";

export type PaneId = "general" | "shortcuts" | "about";

/** The preferences the General pane reads and writes (per account, decision 17). */
export type PrefField = "theme" | "reduced_motion" | "show_completed";

export const PANES: { id: PaneId; label: string; icon: string }[] = [
  { id: "general", label: "General", icon: "tune" },
  { id: "shortcuts", label: "Shortcuts", icon: "keyboard" },
  { id: "about", label: "About", icon: "info" },
];

/** One searchable setting row. Its haystack is title, description and keywords. */
export interface SearchRow {
  id: string;
  pane: PaneId;
  group: string;
  title: string;
  description: string;
  keywords: string;
}

/** Whitespace-separated words of the query, lower-cased. Empty means no search. */
export function queryTokens(query: string): string[] {
  return query.trim().toLowerCase().split(/\s+/).filter((token) => token !== "");
}

/** A row matches when every word is part of its text (case-insensitive). */
export function rowMatches(row: SearchRow, tokens: string[]): boolean {
  const haystack = `${row.title} ${row.description} ${row.keywords}`.toLowerCase();
  return tokens.every((token) => haystack.includes(token));
}

export function cx(...parts: (string | false | null | undefined)[]): string {
  return parts.filter(Boolean).join(" ");
}

/** The search well: a leading glyph, the query field and a clear button while the query is not empty. */
export function SearchBox({
  value,
  onValueChange,
  onClear,
}: {
  value: string;
  onValueChange: (value: string) => void;
  onClear: () => void;
}) {
  const inputRef = React.useRef<HTMLInputElement | null>(null);
  const clear = () => {
    onClear();
    inputRef.current?.focus();
  };
  return (
    <div
      className="flex h-[1.75rem] min-w-0 items-center gap-1.5 rounded-(--radius-field) bg-wash-2 pl-[0.643rem] pr-0.5 focus-within:ring-2 focus-within:ring-focus-1"
      onMouseDown={(event) => {
        if ((event.target as Element).closest("button, input")) return;
        event.preventDefault();
        inputRef.current?.focus();
      }}
    >
      <Icon name="search" size={14} tone="muted" />
      <Input
        ref={inputRef}
        data-part="app.settingsgeneral.search"
        ariaLabel="Search settings"
        placeholder="Search"
        size="sm"
        mode="bare"
        spellCheck={false}
        value={value}
        onValueChange={onValueChange}
        onCancel={() => {
          if (value !== "") onClear();
        }}
        className="min-w-0 flex-1"
      />
      <Button
        data-part="app.settingsgeneral.clearSearch"
        variant="ghost"
        size="sm"
        tone="muted"
        shape="icon"
        icon="close"
        iconSize={14}
        ariaLabel="Clear search"
        description="Clear search"
        retainFocus
        onClick={clear}
        className={cx("shrink-0", value === "" && "hidden")}
      />
    </div>
  );
}

/** A card of rows under an eyebrow label. The label gets the group's clear control while it has changes. */
export function SettingsGroup({
  label,
  labelPart,
  groupPart,
  discardPart,
  changed,
  onDiscard,
  visible = true,
  children,
}: {
  label: string;
  labelPart: string;
  groupPart: string;
  discardPart: string;
  changed: boolean;
  onDiscard: () => void;
  visible?: boolean;
  children?: React.ReactNode;
}) {
  return (
    <section className={cx("flex min-w-0 flex-col gap-2", !visible && "hidden")}>
      <div className="flex min-w-0 items-center gap-2">
        <Text data-part={labelPart} as="div" size="xs" weight="semibold" tone="muted" case="uppercase">
          {label}
        </Text>
        {changed && <span aria-hidden="true" className="block size-[0.429rem] shrink-0 rounded-full bg-content-1" />}
        <span aria-hidden="true" className="block h-px min-w-4 flex-1 bg-edge-2" />
        <Button
          data-part={discardPart}
          variant="ghost"
          size="sm"
          tone="muted"
          shape="icon"
          icon="undo"
          iconSize={14}
          ariaLabel="Discard changes in this group"
          description="Discard changes in this group"
          onClick={onDiscard}
          className={cx("shrink-0", !changed && "hidden")}
        />
      </div>
      <div
        data-part={groupPart}
        className="divide-y divide-edge-1 overflow-hidden rounded-(--radius-panel) border border-edge-2 bg-surface-2"
      >
        {children}
      </div>
    </section>
  );
}

/** One setting: title and description, with its control inline on the right or stacked under the text. */
export function SettingItem({
  title,
  titlePart,
  description,
  descriptionPart,
  visible = true,
  stacked = false,
  error = "",
  children,
}: {
  title: string;
  titlePart: string;
  description: string;
  descriptionPart: string;
  visible?: boolean;
  stacked?: boolean;
  error?: string;
  children?: React.ReactNode;
}) {
  return (
    <div className={cx("flex min-w-0 flex-col px-[0.571rem] py-[0.5rem]", !visible && "hidden")}>
      <div className="flex min-w-0 items-center gap-3">
        <div className="flex min-w-0 flex-1 flex-col gap-[0.143rem]">
          <Text data-part={titlePart} as="div" size="md" tone="primary" className="leading-[1.393rem]">
            {title}
          </Text>
          <Text
            data-part={descriptionPart}
            as="p"
            size="xs"
            tone="muted"
            className="text-[0.821rem] leading-[1.232rem]"
          >
            {description}
          </Text>
        </div>
        {!stacked && <div className="flex shrink-0 items-center">{children}</div>}
      </div>
      {stacked && <div className="mt-[0.357rem] flex min-w-0">{children}</div>}
      {error !== "" && (
        <Text as="p" role="status" size="xs" tone="danger" className="mt-1">
          {error}
        </Text>
      )}
    </div>
  );
}

export interface Choice<V extends string> {
  value: V;
  label: string;
  part: string;
  icon?: string;
}

/** A tray of segments; the selected one is raised and aria-pressed. Each segment is a plain button. */
export function Segments<V extends string>({
  label,
  value,
  choices,
  onChange,
  stacked = false,
}: {
  label: string;
  value: V;
  choices: Choice<V>[];
  onChange: (value: V) => void;
  stacked?: boolean;
}) {
  return (
    <div
      role="group"
      aria-label={label}
      className={cx(
        "flex h-[1.857rem] items-stretch gap-[0.143rem] rounded-(--radius-field) bg-wash-2 p-[0.143rem]",
        stacked ? "w-full" : "shrink-0",
      )}
    >
      {choices.map((choice) => {
        const selected = choice.value === value;
        return (
          <Button
            key={choice.value}
            data-part={choice.part}
            variant="ghost"
            size="md"
            tone="default"
            width={stacked ? "full" : "content"}
            label={choice.label}
            ariaLabel={choice.label}
            icon={choice.icon ?? ""}
            iconSize={15}
            active={selected}
            aria-pressed={selected}
            onClick={() => onChange(choice.value)}
            className={cx(
              "h-full",
              stacked ? "flex-1 px-2" : "px-2.5",
              "text-[0.857rem] font-semibold",
              selected ? "bg-surface-3 text-content-1 shadow-lift" : "text-content-2 hover:text-content-1",
            )}
          />
        );
      })}
    </div>
  );
}

/** Shown when a search matches nothing. */
export function EmptyResults({ query }: { query: string }) {
  return (
    <div className="flex min-w-0 flex-col items-center pt-[4.5rem] text-center">
      <Icon name="search" size={30} tone="muted" />
      <Text
        data-part="app.settingsgeneral.emptyTitle"
        as="div"
        size="md"
        weight="medium"
        tone="secondary"
        className="mt-2 text-[1.071rem]"
      >
        No settings match
      </Text>
      <Text data-part="app.settingsgeneral.emptySubtitle" as="div" size="caption" tone="muted" className="mt-1">
        {`Nothing here for “${query}”.`}
      </Text>
    </div>
  );
}

/** The scrolling content block of the active pane. It fades in on each mount (each pane switch). */
export function PaneFrame({ children }: { children?: React.ReactNode }) {
  const [shown, setShown] = React.useState(false);
  React.useEffect(() => {
    const frame = window.requestAnimationFrame(() => setShown(true));
    return () => window.cancelAnimationFrame(frame);
  }, []);
  return (
    <div
      role="tabpanel"
      data-part="app.settingsgeneral.contentPanel"
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
