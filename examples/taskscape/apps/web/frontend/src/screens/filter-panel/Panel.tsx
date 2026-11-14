// The Filter & Sort panel that sits in the preview column, and the preview column's empty state.
import { useState } from "react";
import { Button } from "@taskscape/ui/button";
import { EmptyState } from "@taskscape/ui/empty-state";
import { Heading } from "@taskscape/ui/heading";
import { Icon } from "@taskscape/ui/icon";
import { Menu } from "@taskscape/ui/menu";
import { MenuItem } from "@taskscape/ui/menu-item";
import { NumberField } from "@taskscape/ui/number-field";
import { Text } from "@taskscape/ui/text";
import {
  clampDays,
  refineActive,
  sortActive,
  type DateField,
  type DateRange,
  type ShowMode,
  type SortMode,
  type Tri,
  type View,
} from "./model";

export interface Segment<T extends string> {
  value: T;
  label: string;
  part: string;
}

const SORTS: Array<{ value: SortMode; label: string; part: string }> = [
  { value: "manual", label: "Manual", part: "sortManual" },
  { value: "created", label: "Date created", part: "sortCreated" },
  { value: "updated", label: "Date updated", part: "sortUpdated" },
  { value: "alpha", label: "Alphabetical", part: "sortAlphabetical" },
  { value: "done", label: "Done last", part: "sortDoneLast" },
];

const SHOWS: Segment<ShowMode>[] = [
  { value: "all", label: "All", part: "showAll" },
  { value: "active", label: "Active", part: "showActive" },
  { value: "done", label: "Done", part: "showDone" },
];

const DATE_FIELDS: Segment<DateField>[] = [
  { value: "created", label: "Created", part: "dateCreated" },
  { value: "updated", label: "Updated", part: "dateUpdated" },
];

const RANGES: Array<{ value: DateRange; label: string; part: string; patch: Partial<View> }> = [
  { value: "any", label: "Any time", part: "rangeAnyTime", patch: { dateRange: "any" } },
  { value: "24h", label: "Last 24 hours", part: "rangeLast24Hours", patch: { dateRange: "24h" } },
  { value: "7d", label: "Last 7 days", part: "rangeLast7Days", patch: { dateRange: "7d" } },
  { value: "30d", label: "Last 30 days", part: "rangeLast30Days", patch: { dateRange: "30d" } },
  { value: "custom", label: "Custom…", part: "rangeCustom", patch: { dateRange: "custom" } },
];

function triOptions(prefix: string): Segment<Tri>[] {
  return [
    { value: "any", label: "Any", part: `${prefix}Any` },
    { value: "has", label: "With", part: `${prefix}With` },
    { value: "none", label: "Without", part: `${prefix}Without` },
  ];
}

/** A track of choices. Every choice is a button with aria-selected, so each one has its own part. */
function Segments<T extends string>({
  label,
  options,
  value,
  onChange,
  className = "",
}: {
  label: string;
  options: Segment<T>[];
  value: T;
  onChange: (v: T) => void;
  className?: string;
}) {
  return (
    <div
      role="group"
      aria-label={label}
      className={`flex items-stretch gap-px rounded-(--radius-field) bg-wash-2 p-[0.143rem] ${className}`}
    >
      {options.map((o) => {
        const on = o.value === value;
        return (
          <Button
            key={o.value}
            data-part={`app.filterpanel.${o.part}`}
            variant={on ? "primary" : "ghost"}
            size="md"
            tone="default"
            width="full"
            label={o.label}
            aria-selected={on}
            aria-pressed={on}
            className="min-w-0 flex-1 px-1 font-semibold"
            onClick={() => onChange(o.value)}
          />
        );
      })}
    </div>
  );
}

/** A section title: the heading, the changed dot and the clear control while the group is non-default. */
function SectionTitle({
  label,
  part,
  on,
  clearPart,
  clearLabel,
  onClear,
}: {
  label: string;
  part: string;
  on: boolean;
  clearPart: string;
  clearLabel: string;
  onClear: () => void;
}) {
  return (
    <div className="flex min-w-0 items-center gap-2">
      <Text
        as="div"
        role="heading"
        aria-level={3}
        data-part={`app.filterpanel.${part}`}
        size="xs"
        weight="semibold"
        tone="muted"
        overflow="truncate"
        case="uppercase"
        className="shrink-0 tracking-[0.08em]"
      >
        {label}
      </Text>
      {on && <span aria-hidden="true" className="block size-[0.429rem] shrink-0 rounded-full bg-content-1" />}
      <span aria-hidden="true" className="block h-px min-w-4 flex-1 bg-edge-2" />
      {on && (
        <Button
          data-part={`app.filterpanel.${clearPart}`}
          variant="ghost"
          size="sm"
          shape="icon"
          tone="muted"
          icon="restart_alt"
          ariaLabel={clearLabel}
          onClick={onClear}
        />
      )}
    </div>
  );
}

function TriRow({
  icon,
  label,
  prefix,
  value,
  onChange,
}: {
  icon: string;
  label: string;
  prefix: string;
  value: Tri;
  onChange: (v: Tri) => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <span className="flex w-[6rem] shrink-0 items-center gap-2">
        <Icon name={icon} size="sm" tone="muted" />
        <Text size="sm" weight="medium" tone={value === "any" ? "secondary" : "primary"} overflow="truncate">
          {label}
        </Text>
      </span>
      <Segments label={label} options={triOptions(prefix)} value={value} onChange={onChange} className="min-w-0 flex-1" />
    </div>
  );
}

export interface PanelProps {
  subtitle: string;
  view: View;
  resetEnabled: boolean;
  summary: string;
  onPatch: (p: Partial<View>) => void;
  onReset: () => void;
  onClose: () => void;
  onDone: () => void;
}

export function Panel({ subtitle, view, resetEnabled, summary, onPatch, onReset, onClose, onDone }: PanelProps) {
  const [sortOpen, setSortOpen] = useState(false);
  const [rangeOpen, setRangeOpen] = useState(false);
  const sortLabel = SORTS.find((s) => s.value === view.sort)?.label ?? "Manual";
  const rangeLabel = RANGES.find((r) => r.value === view.dateRange)?.label ?? "Any time";
  const manual = view.sort === "manual";
  const sortOn = sortActive(view);
  const refineOn = refineActive(view);
  const dateOn = view.dateRange !== "any";
  const showOn = view.show !== "all";

  return (
    <div className="flex h-full min-h-0 flex-col bg-surface-2">
      <div className="flex h-[3.143rem] shrink-0 items-center gap-2 border-b border-edge-1 px-3">
        <Icon name="tune" size="md" tone="muted" />
        <div className="flex min-w-0 flex-1 flex-col">
          <Heading size="sm" data-part="app.filterpanel.panelTitle">
            Filter & Sort
          </Heading>
          <Text
            as="span"
            size="xs"
            weight="medium"
            tone="muted"
            overflow="truncate"
            data-part="app.filterpanel.panelSubtitle"
          >
            {subtitle}
          </Text>
        </div>
        <Button
          data-part="app.filterpanel.closeButton"
          variant="ghost"
          tone="muted"
          size="md"
          shape="icon"
          icon="close"
          ariaLabel="Close"
          onClick={onClose}
        />
      </div>

      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-3 py-3">
        <div className="flex flex-col gap-2">
          <SectionTitle
            label="Sort"
            part="sectionSort"
            on={sortOn}
            clearPart="clearSort"
            clearLabel="Clear sort"
            onClear={() => onPatch({ sort: "manual", dir: "asc" })}
          />
          <div className="flex items-center gap-1">
            <Menu
              open={sortOpen}
              onOpenChange={setSortOpen}
              placement="control"
              width="content"
              ariaLabel="Sort"
              trigger={
                <Button
                  data-part="app.filterpanel.sortMenu"
                  variant="neutral"
                  size="md"
                  width="full"
                  icon={manual ? "drag_indicator" : "sort"}
                  trailingIcon="unfold_more"
                  label={sortLabel}
                  className="min-w-0 flex-1 justify-start"
                />
              }
            >
              {SORTS.map((s) => (
                <MenuItem
                  key={s.value}
                  data-part={`app.filterpanel.${s.part}`}
                  label={s.label}
                  onSelect={() => onPatch({ sort: s.value })}
                />
              ))}
            </Menu>
            <div
              role="group"
              aria-label="Direction"
              className="flex shrink-0 items-center gap-px rounded-(--radius-field) bg-surface-0 p-[0.143rem]"
            >
              <Button
                data-part="app.filterpanel.sortAscending"
                variant={!manual && view.dir === "asc" ? "primary" : "ghost"}
                size="sm"
                shape="icon"
                icon="arrow_upward"
                ariaLabel="Ascending"
                disabled={manual}
                aria-selected={!manual && view.dir === "asc"}
                aria-pressed={!manual && view.dir === "asc"}
                onClick={() => onPatch({ dir: "asc" })}
              />
              <Button
                data-part="app.filterpanel.sortDescending"
                variant={!manual && view.dir === "desc" ? "primary" : "ghost"}
                size="sm"
                shape="icon"
                icon="arrow_downward"
                ariaLabel="Descending"
                disabled={manual}
                aria-selected={!manual && view.dir === "desc"}
                aria-pressed={!manual && view.dir === "desc"}
                onClick={() => onPatch({ dir: "desc" })}
              />
            </div>
          </div>
        </div>

        <div className="flex flex-col gap-2">
          <SectionTitle
            label="Show"
            part="sectionShow"
            on={showOn}
            clearPart="clearShow"
            clearLabel="Clear show"
            onClear={() => onPatch({ show: "all" })}
          />
          <Segments label="Show" options={SHOWS} value={view.show} onChange={(v) => onPatch({ show: v })} />
        </div>

        <div className="flex flex-col gap-2">
          <SectionTitle
            label="Refine"
            part="sectionRefine"
            on={refineOn}
            clearPart="clearRefine"
            clearLabel="Clear refine"
            onClear={() => onPatch({ notes: "any", attachments: "any", subtasks: "any" })}
          />
          <TriRow icon="notes" label="Notes" prefix="notes" value={view.notes} onChange={(v) => onPatch({ notes: v })} />
          <TriRow
            icon="attach_file"
            label="Attachments"
            prefix="attachments"
            value={view.attachments}
            onChange={(v) => onPatch({ attachments: v })}
          />
          <TriRow
            icon="account_tree"
            label="Subtasks"
            prefix="subtasks"
            value={view.subtasks}
            onChange={(v) => onPatch({ subtasks: v })}
          />
        </div>

        <div className="flex flex-col gap-2">
          <SectionTitle
            label="Date"
            part="sectionDate"
            on={dateOn}
            clearPart="clearDate"
            clearLabel="Clear date"
            onClear={() => onPatch({ dateRange: "any", dateField: "created", days: 7 })}
          />
          <Segments
            label="Date field"
            options={DATE_FIELDS}
            value={view.dateField}
            onChange={(v) => onPatch({ dateField: v })}
          />
          <Menu
            open={rangeOpen}
            onOpenChange={setRangeOpen}
            placement="control"
            width="content"
            ariaLabel="Date range"
            trigger={
              <Button
                data-part="app.filterpanel.dateRangeButton"
                variant="neutral"
                size="md"
                width="full"
                icon="schedule"
                trailingIcon="unfold_more"
                label={rangeLabel}
                className="justify-start"
              />
            }
          >
            {RANGES.map((r) => (
              <MenuItem
                key={r.value}
                data-part={`app.filterpanel.${r.part}`}
                label={r.label}
                onSelect={() => onPatch(r.patch)}
              />
            ))}
          </Menu>
          {view.dateRange === "custom" && (
            <div className="flex items-center gap-2">
              <Text
                size="sm"
                weight="medium"
                tone="secondary"
                data-part="app.filterpanel.daysCaption"
                overflow="truncate"
              >
                Within the last
              </Text>
              <Button
                data-part="app.filterpanel.daysFewer"
                variant="neutral"
                size="sm"
                shape="icon"
                icon="remove"
                ariaLabel="Fewer days"
                onClick={() => onPatch({ days: clampDays(view.days - 1) })}
              />
              <NumberField
                data-part="app.filterpanel.daysInput"
                ariaLabel="Days"
                value={view.days}
                onValueChange={(n) => onPatch({ days: n })}
                onChange={(e) => {
                  // An emptied field becomes 1 at once (F05-017).
                  if (e.target.value === "") onPatch({ days: 1 });
                }}
              />
              <Button
                data-part="app.filterpanel.daysMore"
                variant="neutral"
                size="sm"
                shape="icon"
                icon="add"
                ariaLabel="More days"
                onClick={() => onPatch({ days: clampDays(view.days + 1) })}
              />
              <Text size="sm" weight="medium" tone="secondary" data-part="app.filterpanel.daysUnit">
                days
              </Text>
            </div>
          )}
        </div>
      </div>

      <div className="flex h-[2.286rem] shrink-0 items-center gap-2 border-t border-edge-1 px-3">
        <span
          aria-hidden="true"
          className={`block size-[0.429rem] shrink-0 rounded-full ${resetEnabled ? "bg-accent-500" : "bg-edge-3"}`}
        />
        <Text
          role="status"
          data-part="app.filterpanel.summary"
          size="xs"
          weight="medium"
          tone="secondary"
          numeric="tabular"
          overflow="truncate"
        >
          {summary}
        </Text>
      </div>

      <div className="flex h-[3.357rem] shrink-0 items-center gap-2 border-t border-edge-1 bg-surface-1 px-3">
        <Button
          data-part="app.filterpanel.resetButton"
          variant="ghost"
          size="lg"
          icon="restart_alt"
          label="Reset"
          disabled={!resetEnabled}
          onClick={onReset}
        />
        <Button
          data-part="app.filterpanel.doneButton"
          variant="primary"
          size="xl"
          label="Done"
          className="ml-auto"
          onClick={onDone}
        />
      </div>
    </div>
  );
}

/** The preview column's empty state, shown while the Filter & Sort panel is closed. */
export function PreviewEmpty() {
  return (
    <div className="flex h-full min-h-0 flex-col items-center justify-center bg-surface-2">
      <EmptyState
        data-part="app.filterpanel.previewEmpty"
        role="status"
        scope="list"
        icon="ads_click"
        title="No task selected"
        subtitle="Select a task to inspect it"
        className="pb-0 pt-0"
      />
    </div>
  );
}
