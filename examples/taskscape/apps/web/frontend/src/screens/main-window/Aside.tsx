// The right aside of the main window: the trash, the filter panel, or the preview panel. The preview
// panel shows the multi-select inspector for two or more picks, the task inspector for the previewed
// task, the draft inspector for a new task, and otherwise the empty preview.
import { Icon } from "@taskscape/ui/icon";
import { Text } from "@taskscape/ui/text";
import type { Id } from "../../api/types";
import { useUi, type Aside as AsideMode } from "../../state/ui";
import { DraftInspector } from "../draft-inspector/DraftInspector";
import { FilterPanel } from "../filter-panel/FilterPanel";
import { MultiInspector } from "../multi-inspector/MultiInspector";
import { TaskInspector } from "../task-inspector/TaskInspector";
import { TrashPane } from "../trash/TrashPane";

/** The aside width classes, one per 14 px (1rem) step from 280 px to 420 px. */
const ASIDE_WIDTHS = [
  "w-[20rem]", "w-[21rem]", "w-[22rem]", "w-[23rem]", "w-[24rem]", "w-[25rem]",
  "w-[26rem]", "w-[27rem]", "w-[28rem]", "w-[29rem]", "w-[30rem]",
];

export function asideWidthClass(px: number): string {
  const step = Math.round((px - 280) / 14);
  return ASIDE_WIDTHS[Math.min(ASIDE_WIDTHS.length - 1, Math.max(0, step))] ?? "w-[22rem]";
}

export interface AsideProps {
  mode: AsideMode;
  width: number;
  picked: number;
  previewId: Id | null;
  draftListId: Id | null;
  draftListName: string | null;
  projectName: string | null;
}

export function Aside({ mode, width, picked, previewId, draftListId, draftListName, projectName }: AsideProps) {
  const setAside = useUi((s) => s.setAside);
  const setDraft = useUi((s) => s.setDraft);

  let body;
  if (mode === "trash") {
    body = (
      <div data-part="app.mainwindow.trashPane" className="flex min-h-0 flex-1 flex-col">
        <TrashPane onClose={() => setAside("preview")} />
      </div>
    );
  } else if (mode === "filter") {
    body = (
      // The filter panel is a whole window of its own (its title bar, list tabs and task panes come with it). Here
      // only its Filter & Sort column shows: the window's own title bar and its panes are hidden from this aside.
      <div data-part="app.mainwindow.filterPanel" className="flex min-h-0 flex-1 flex-col [&_[data-slot=titleBar]]:hidden [&_[data-slot=body]>div:first-child]:hidden">
        <FilterPanel />
      </div>
    );
  } else {
    body = (
      <div data-part="app.mainwindow.previewPanel" className="flex min-h-0 flex-1 flex-col">
        {picked > 1 ? (
          <div data-part="app.mainwindow.multiInspector" className="flex min-h-0 flex-1 flex-col">
            <MultiInspector />
          </div>
        ) : previewId != null ? (
          <TaskInspector />
        ) : draftListId != null ? (
          <div data-part="app.mainwindow.draftInspector" className="flex min-h-0 flex-1 flex-col">
            <DraftInspector
              listId={draftListId}
              projectName={projectName}
              listName={draftListName}
              onAction={() => setDraft(null)}
            />
          </div>
        ) : (
          <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-1 px-4 text-center">
            <Icon name="ads_click" size={28} tone="muted" />
            <Text data-part="app.mainwindow.previewEmpty" as="div" tone="secondary" className="mt-2 font-display text-[1.071rem] font-medium">
              No task selected
            </Text>
            <Text as="div" tone="muted" className="text-[0.893rem]">
              Select a task to inspect it
            </Text>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className={`flex min-h-0 shrink-0 flex-col overflow-hidden border-l border-edge-2 bg-surface-1 ${asideWidthClass(width)}`}>
      {body}
    </div>
  );
}
