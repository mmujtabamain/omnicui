// Filter & Sort as it sits in the main window's aside: the panel alone, at the aside's full width,
// editing the shared per-list views (useUi.views) that the task list applies. FilterPanel draws the
// same panel inside a whole window of its own, for its contract.
import { useLists, useTasks } from "../../api/queries";
import type { Id } from "../../api/types";
import { useUi } from "../../state/ui";
import { DEFAULT_VIEW, computePane, viewActive, type View } from "./model";
import { Panel } from "./Panel";

export function FilterAside() {
  const { projectId, panes, views, patchViews, resetViews, setAside } = useUi();
  const lists = useLists(projectId).data ?? [];
  const paneA = useTasks({ list: panes[0] ?? null });
  const paneB = useTasks({ list: panes[1] ?? null });
  const paneTasks = [paneA.data ?? [], paneB.data ?? []];
  const now = Date.now();

  // The panel edits every list on screen as one (both panes in split view).
  const named = panes
    .map((id, i) => ({ id, list: lists.find((l) => l.id === id), tasks: paneTasks[i] ?? [] }))
    .filter((p): p is { id: Id; list: NonNullable<typeof p.list>; tasks: typeof paneTasks[number] } => p.id != null && p.list !== undefined);
  const viewOfList = (id: Id): View => views[id] ?? DEFAULT_VIEW;
  const view = named[0] ? viewOfList(named[0].id) : DEFAULT_VIEW;
  const counts = named.map((p) => {
    const m = computePane(p.tasks, viewOfList(p.id), { query: "", fields: "both", showCompleted: true, now }, {});
    return { name: p.list.name, visible: m.visible, total: m.total };
  });
  let summary = "Nothing to filter";
  if (counts.length > 1) summary = counts.map((c) => `${c.name} ${c.visible}/${c.total}`).join(" · ");
  else if (counts[0]) summary = `Showing ${counts[0].visible} of ${counts[0].total} tasks`;
  const ids = named.map((p) => p.id);

  return (
    <div className="flex min-h-0 w-full flex-1 flex-col">
      <Panel
        subtitle={named.map((p) => p.list.name).join(", ") || "No list open"}
        view={view}
        resetEnabled={named.some((p) => viewActive(viewOfList(p.id)))}
        summary={summary}
        onPatch={(patch) => patchViews(ids, patch)}
        onReset={() => resetViews(ids)}
        onClose={() => setAside("preview")}
        onDone={() => setAside("preview")}
      />
    </div>
  );
}
