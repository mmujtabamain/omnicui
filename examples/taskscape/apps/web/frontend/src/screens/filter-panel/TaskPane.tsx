// One task list pane: the search well, the rows of the list (or the matches grouped by list), the empty
// states and the footer with the done count.
import { useState, type Ref } from "react";
import { Badge } from "@taskscape/ui/badge";
import { Button } from "@taskscape/ui/button";
import { Checkbox } from "@taskscape/ui/checkbox";
import { EmptyState } from "@taskscape/ui/empty-state";
import { Heading } from "@taskscape/ui/heading";
import { Icon } from "@taskscape/ui/icon";
import { Input } from "@taskscape/ui/input";
import { Menu } from "@taskscape/ui/menu";
import { MenuItem } from "@taskscape/ui/menu-item";
import { ProgressBar } from "@taskscape/ui/progress-bar";
import { Row } from "@taskscape/ui/row";
import { Text } from "@taskscape/ui/text";
import type { Id } from "../../api/types";
import { useUi } from "../../state/ui";
import type { Fields, PaneView, RowModel, Scope } from "./model";

/** Contract parts on the rows of the first list (Errands in the standard data set). */
const ROW_PARTS: Record<string, string> = {
  "Pick up prescription": "rowPickUp",
  "Book bike service": "rowBike",
  "Buy groceries for the week": "rowGroceries",
  "Sourdough loaf": "rowSourdough",
  "Oat milk": "rowOatMilk",
  "Post the parcel to Mum": "rowPost",
  "Return library books": "rowReturnLibrary",
};

const GROUP_PARTS: Record<string, string> = {
  Errands: "groupHeadingErrands",
  Reading: "groupHeadingReading",
};

const part = (on: boolean, name: string) => (on ? { "data-part": `app.filterpanel.${name}` } : {});

export interface TaskPaneProps {
  pane: PaneView;
  /** The first pane carries the contract parts. */
  primary: boolean;
  split: boolean;
  inputRef?: Ref<HTMLInputElement | HTMLTextAreaElement>;
  panelActive: boolean;
  scope: Scope;
  fields: Fields;
  onQuery: (q: string) => void;
  onScope: (s: Scope) => void;
  onFields: (f: Fields) => void;
  onToggle: (id: Id) => void;
  onDone: (id: Id, done: boolean) => void;
  onOpenPanel: () => void;
}

export function TaskPane(p: TaskPaneProps) {
  const { pane, primary } = p;
  const [optionsOpen, setOptionsOpen] = useState(false);
  const name = pane.list?.name ?? "";
  const searching = pane.query.trim() !== "";
  const flat = pane.sections.flatMap((s) => s.rows);
  const firstRow = flat[0]?.task.id;
  const firstParent = flat.find((m) => m.hasKids)?.task.id;
  const firstDone = flat.find((m) => m.task.done)?.task.id;
  const pct = pane.total > 0 ? Math.round((pane.done / pane.total) * 100) : 0;

  const renderRow = (m: RowModel) => {
    const t = m.task;
    const isFirst = primary && t.id === firstRow;
    const named = primary && ROW_PARTS[t.title] ? { "data-part": `app.filterpanel.${ROW_PARTS[t.title]}` } : {};
    const note = t.note_items[0]?.content ?? t.notes;
    return (
      <div key={t.id} {...part(isFirst, "row")}>
        <Row
          role="listitem"
          density="md"
          indent={m.depth * 24}
          className="h-auto min-h-[2.25rem] py-1"
          {...named}
        >
          <Checkbox
            intent="done"
            shape="round"
            size="sm"
            checked={t.done}
            ariaLabel="Select"
            onCheckedChange={(next) => p.onDone(t.id, next)}
            {...part(primary && t.id === firstDone, "doneCheck")}
          />
          <div className="flex min-w-0 flex-1 flex-col">
            <Button
              variant="ghost"
              tone={t.done ? "muted" : "strong"}
              size="default"
              width="full"
              label={t.title}
              className={`h-auto min-h-0 justify-start px-0 text-[1rem] leading-[1.1875rem] ${t.done ? "line-through" : ""}`}
              onClick={() => useUi.getState().preview(t.id)}
            />
            {note !== "" && (
              <Text as="span" size="xs" tone="secondary" overflow="truncate" className="block">
                {note}
              </Text>
            )}
          </div>
          {m.hasKids && !searching && (
            <Button
              {...part(t.id === firstParent, "subtaskToggle")}
              variant="ghost"
              size="sm"
              shape="icon"
              tone="muted"
              icon="chevron_right"
              ariaLabel={m.expanded ? "Collapse" : "Expand"}
              reveal={m.expanded ? "row" : "always"}
              className={m.expanded ? "rotate-90" : undefined}
              onClick={() => p.onToggle(t.id)}
            />
          )}
          {m.hasKids && !searching && m.kidsVisible > 0 && (
            <Badge {...part(t.id === firstParent, "subtaskCount")} tone="outline" size="md">
              {`${m.kidsDone} / ${m.kidsVisible}`}
            </Badge>
          )}
          <Button
            {...part(isFirst, "rowMore")}
            variant="ghost"
            size="sm"
            shape="icon"
            tone="muted"
            icon="more_horiz"
            ariaLabel="More"
          />
        </Row>
      </div>
    );
  };

  const hasRows = flat.length > 0;

  return (
    <div className={`flex min-h-0 min-w-0 flex-1 flex-col ${p.split ? "border-r border-edge-1" : ""}`}>
      <div className="flex shrink-0 items-center gap-2 px-4 pb-2 pt-2">
        <div className="flex h-[2.5rem] min-w-0 flex-1 items-center gap-1 rounded-full bg-wash-2 pl-3 pr-1 focus-within:ring-2 focus-within:ring-focus-1">
          <Icon name="search" size="md" tone="muted" />
          <Input
            {...part(primary, "searchField")}
            ref={p.inputRef}
            mode="bare"
            size="md"
            value={pane.query}
            placeholder={name !== "" ? `Search ${name}` : "Search"}
            ariaLabel="Search tasks"
            onValueChange={p.onQuery}
            onCancel={() => p.onQuery("")}
            className="min-w-0 flex-1"
          />
          {searching && (
            <Text
              {...part(primary, "searchCount")}
              as="span"
              size="xs"
              weight="semibold"
              tone="muted"
              numeric="tabular"
            >
              {String(pane.matches)}
            </Text>
          )}
          {searching && (
            <Button
              {...part(primary, "searchClear")}
              variant="ghost"
              size="sm"
              shape="icon"
              tone="muted"
              icon="close"
              ariaLabel="Clear search"
              onClick={() => p.onQuery("")}
            />
          )}
          <Menu
            open={optionsOpen}
            onOpenChange={setOptionsOpen}
            placement="control"
            width="content"
            ariaLabel="Search options"
            trigger={
              <Button
                {...part(primary, "searchOptions")}
                variant="ghost"
                size="sm"
                shape="icon"
                tone="muted"
                icon="manage_search"
                ariaLabel="Search options"
              />
            }
          >
            <MenuItem label="This list" onSelect={() => p.onScope("list")} />
            <MenuItem {...part(primary, "scopeWholeProject")} label="Whole project" onSelect={() => p.onScope("project")} />
            <MenuItem label="All projects" onSelect={() => p.onScope("all")} />
            <MenuItem label="Title & notes" onSelect={() => p.onFields("both")} />
            <MenuItem label="Title only" onSelect={() => p.onFields("title")} />
            <MenuItem {...part(primary, "fieldsNotesOnly")} label="Notes only" onSelect={() => p.onFields("notes")} />
          </Menu>
          <span aria-hidden="true" className="block h-5 w-px shrink-0 bg-edge-2" />
          <Button
            {...part(primary, "newTask")}
            variant="ghost"
            size="sm"
            shape="icon"
            tone="muted"
            icon="add"
            ariaLabel="New task"
          />
        </div>
      </div>

      <div role="list" className="flex min-h-0 flex-1 flex-col overflow-y-auto">
        {pane.sections.map((s) => (
          <div key={s.key} className="flex flex-col">
            {s.heading !== null && (
              <Heading
                as="h3"
                size="sm"
                className="px-4 pb-1 pt-3 text-[0.857rem] text-content-2"
                {...part(primary && GROUP_PARTS[s.listName] !== undefined, GROUP_PARTS[s.listName] ?? "")}
              >
                {s.heading}
              </Heading>
            )}
            {s.rows.map(renderRow)}
          </div>
        ))}
        {!hasRows &&
          (searching ? (
            <div className="flex flex-col items-center">
              <EmptyState
                {...part(primary, "emptyState")}
                role="status"
                scope="list"
                icon="search_off"
                title={`No matches in ${name}`}
                className="pb-2"
              />
              <Text {...part(primary, "noMatchHint")} size="sm" tone="muted" className="pb-10">
                Try a different search or scope
              </Text>
            </div>
          ) : (
            <EmptyState
              {...part(primary, "emptyState")}
              role="status"
              scope="list"
              icon="landscape"
              title="Nothing here yet"
            />
          ))}
      </div>

      <div className="flex h-[2.286rem] shrink-0 items-center gap-2 border-t border-edge-1 bg-surface-2 pl-1 pr-3">
        <Button
          {...part(primary, "sortAndFilter")}
          variant="ghost"
          size="footer"
          tone="muted"
          icon="tune"
          ariaLabel="Sort & filter"
          label={p.panelActive ? "Filtered" : ""}
          onClick={p.onOpenPanel}
        />
        <span aria-hidden="true" className="block h-4 w-px bg-edge-2" />
        <Text
          {...part(primary, "doneCount")}
          as="span"
          size="xs"
          weight="semibold"
          tone="muted"
          case="uppercase"
          numeric="tabular"
        >
          {`${pane.done}/${pane.total} done`}
        </Text>
        <ProgressBar
          {...part(primary, "progressBar")}
          className="ml-auto"
          value={pane.done}
          max={Math.max(pane.total, 1)}
          ariaLabel="Done progress"
        />
        <Text {...part(primary, "donePercent")} as="span" size="xs" weight="semibold" tone="secondary" numeric="tabular">
          {`${pct}%`}
        </Text>
      </div>
    </div>
  );
}
