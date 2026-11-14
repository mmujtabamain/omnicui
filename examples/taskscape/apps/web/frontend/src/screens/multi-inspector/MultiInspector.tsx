// The multi-select inspector: the body of the preview panel while two or more tasks are selected in
// the focused pane. It shows the count, the done tally and one row per selected task. There is no
// verb grid (decision 6): Done, Undone, Move, Copy and Delete live in the bulk bar only.
import { Badge } from "@taskscape/ui/badge";
import { Button } from "@taskscape/ui/button";
import { SectionHeader } from "@taskscape/ui/section-header";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { useLists, useTasks, useWrite } from "../../api/queries";
import { useUi } from "../../state/ui";
import { SelectedRow, type RowSlot } from "./SelectedRow";

const PARCEL_TITLE = "Post the parcel to Mum";

export function MultiInspector() {
  const listId = useUi((s) => s.panes[s.activePane] ?? null);
  const projectId = useUi((s) => s.projectId);
  const selection = useUi((s) => s.selection);
  const previewId = useUi((s) => s.previewId);
  const select = useUi((s) => s.select);
  const preview = useUi((s) => s.preview);
  const tasks = useTasks({ list: listId });
  const lists = useLists(projectId);
  const update = useWrite(ep.tasks.update);

  // The selected tasks in list order. Tasks not in the focused list are not shown.
  const selected = (tasks.data ?? [])
    .filter((t) => selection.includes(t.id))
    .sort((a, b) => a.sort_order - b.sort_order);

  // Fewer than two selected tasks: the single-task inspector takes over, so this screen is not shown.
  if (selected.length < 2) return null;

  const count = selected.length;
  const done = selected.filter((t) => t.done).length;
  const listName = (id: number) => lists.data?.find((l) => l.id === id)?.name;

  const slotOf = (index: number, title: string): RowSlot => {
    if (index === 0) return "first";
    if (title === PARCEL_TITLE) return "parcel";
    if (index === count - 1) return "last";
    return "other";
  };

  return (
    <div
      data-part="app.multiinspector.inspector"
      className="animate-rise flex h-full min-h-0 w-full flex-col overflow-hidden border-l border-edge-1 bg-surface-1"
    >
      <div className="flex shrink-0 items-center gap-space-5 p-space-7">
        <Badge data-part="app.multiinspector.count" tone="accent" shape="round" size="lg" className="font-bold">
          {count}
        </Badge>
        <div className="flex min-w-0 flex-1 flex-col">
          <Text
            as="p"
            data-part="app.multiinspector.headline"
            weight="semibold"
            className="font-display text-[1.143rem] leading-[1.714rem]"
          >
            {`${count} tasks selected`}
          </Text>
          <Text
            as="p"
            data-part="app.multiinspector.tally"
            tone="muted"
            numeric="tabular"
            className="text-[0.821rem] leading-[1.232rem]"
          >
            {`${done} done / ${count}`}
          </Text>
        </div>
        <Button
          data-part="app.multiinspector.clearSelection"
          variant="ghost"
          tone="muted"
          size="sm"
          shape="icon"
          width="content"
          icon="close"
          iconSize={16}
          ariaLabel="Clear selection"
          description="Clear selection"
          onClick={() => {
            select([]);
            preview(null);
          }}
        />
      </div>

      <div
        data-part="app.multiinspector.selectedList"
        className="flex min-h-0 flex-1 flex-col overflow-y-auto px-space-7 pt-[0.714rem] pb-space-7"
      >
        <SectionHeader data-part="app.multiinspector.sectionLabel" label="Selected" as="div" rule="trailing" />
        <div className="-mx-1.5 mt-[0.5rem] flex flex-col">
          {selected.map((task, index) => (
            <SelectedRow
              key={task.id}
              task={task}
              listName={listName(task.list)}
              active={task.id === previewId}
              slot={slotOf(index, task.title)}
              onCheck={(next) => update.mutate([task.id, { done: next }])}
              onPeek={() => preview(task.id)}
              onOpen={() => {
                select([task.id]);
                preview(task.id);
              }}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
