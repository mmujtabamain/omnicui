// The new-task draft in the preview panel: the title field, the breadcrumb and the Start with block.
// Nothing is written until a commit (Enter, Shift+Enter, Tab or a click away with a title), Note or
// Image; Escape and Discard draft only drop the draft. The parent owns whether the draft is shown:
// it is told each action once, and unmounts the draft after it.
import * as React from "react";
import { Button } from "@taskscape/ui/button";
import { Icon } from "@taskscape/ui/icon";
import { Input } from "@taskscape/ui/input";
import { SectionHeader } from "@taskscape/ui/section-header";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { useWrite } from "../../api/queries";
import type { Id } from "../../api/types";
import { useUi } from "../../state/ui";

export type DraftAction = "commit" | "cancel" | "note";

export interface DraftInspectorProps {
  /** The list the task is created in; null when the draft has no target list. */
  listId: Id | null;
  /** The project name shown first in the breadcrumb. */
  projectName: string | null;
  /** The list name shown after "/" in the breadcrumb; omitted when null. */
  listName: string | null;
  /** Reported once per draft action (commit, cancel or note), after the busy guard. */
  onAction?: (action: DraftAction, title: string) => void;
  /** Reported once when an image has been added and its task created. */
  onImageAdded?: () => void;
}

export function DraftInspector({ listId, projectName, listName, onAction, onImageAdded }: DraftInspectorProps) {
  const [title, setTitle] = React.useState("");
  const root = React.useRef<HTMLDivElement>(null);
  // Set by the first action: the draft is closing, so no second action may run (F04-119).
  const busy = React.useRef(false);
  // Set while the native file chooser is open: the title losing focus to it is not a commit.
  const picking = React.useRef(false);

  const createTask = useWrite(ep.tasks.create);
  const uploadFile = useWrite(ep.attachments.upload);

  const run = (action: DraftAction, value: string) => {
    if (busy.current) return;
    busy.current = true;
    if (action === "commit" && value !== "" && listId != null) createTask.mutate([listId, value]);
    // Note creates the task, then opens it in the inspector with a new note ready to type.
    if (action === "note" && listId != null) {
      createTask
        .mutateAsync([listId, value === "" ? "Untitled" : value])
        .then((task) => {
          onAction?.(action, value);
          useUi.getState().previewWithNote(task.id);
        })
        .catch(() => undefined);
      return;
    }
    onAction?.(action, value);
  };

  const addImage = (file: File) => {
    if (busy.current || listId == null) return;
    busy.current = true;
    const name = title.trim() === "" ? "Untitled" : title.trim();
    createTask
      .mutateAsync([listId, name])
      .then((task) => uploadFile.mutate([task.id, file]))
      .catch(() => undefined);
    onImageAdded?.();
  };

  const pickImage = () => {
    if (busy.current) return;
    const chooser = document.createElement("input");
    chooser.type = "file";
    chooser.accept = "image/*";
    chooser.onchange = () => {
      picking.current = false;
      const file = chooser.files?.[0];
      if (file) addImage(file);
    };
    chooser.addEventListener("cancel", () => {
      picking.current = false;
    });
    picking.current = true;
    chooser.click();
  };

  return (
    <div ref={root} className="flex min-h-0 flex-1 flex-col animate-rise">
      <div className="relative p-4 pb-4">
        <span aria-hidden="true" className="absolute left-0 top-[1.143rem] h-4 w-0.5 bg-accent-500" />
        <div className="flex items-start gap-3">
          <Input
            data-part="app.draftinspector.title"
            mode="draft"
            size="display"
            multiline
            autoFocus
            spellCheck={false}
            ariaLabel="Title"
            placeholder="Untitled"
            className="min-w-0 flex-1"
            value={title}
            onValueChange={setTitle}
            onPaste={(e) => {
              const file = Array.from(e.clipboardData.files).find((f) => f.type.startsWith("image/"));
              if (!file) return;
              e.preventDefault();
              addImage(file);
            }}
            onSubmit={(text) => run("commit", text)}
            onCommit={(text) => {
              // Focus leaving the title is a click-away and commits. Focus moving to one of the
              // draft's own controls, or to the file chooser, is not: the draft stays (checked after
              // the focus has moved).
              window.setTimeout(() => {
                const inside = root.current?.contains(document.activeElement) ?? false;
                if (inside || picking.current) return;
                run("commit", text);
              }, 0);
            }}
            onCancel={() => run("cancel", "")}
          />
          <Button
            data-part="app.draftinspector.discardDraft"
            shape="icon"
            size="default"
            tone="muted"
            icon="close"
            ariaLabel="Discard draft"
            description="Discard draft"
            retainFocus
            tabbable={false}
            className="shrink-0"
            onClick={() => run("cancel", "")}
          />
        </div>
        <Text
          data-part="app.draftinspector.breadcrumb"
          as="div"
          size="xs"
          tone="muted"
          className="mt-2 flex items-center gap-1.5"
        >
          <Icon name="folder_open" size={13} weight={300} tone="muted" className="shrink-0" />
          <Text tone="secondary" overflow="truncate">
            {projectName ?? "—"}
          </Text>
          {listName != null && (
            <Text tone="muted" className="shrink-0">
              / {listName}
            </Text>
          )}
        </Text>
      </div>

      <div className="px-4 pb-4">
        <SectionHeader data-part="app.draftinspector.startWith" as="div" label="Start with" rule="trailing" />
        <div className="mt-2 flex gap-1.5">
          <Button
            data-part="app.draftinspector.noteButton"
            variant="dashed"
            size="dashedShort"
            tone="muted"
            icon="add"
            label="Note"
            retainFocus
            tabbable={false}
            className="flex-1"
            onClick={() => run("note", title.trim())}
          />
          <Button
            data-part="app.draftinspector.imageButton"
            variant="dashed"
            size="dashedShort"
            tone="muted"
            icon="image"
            label="Image"
            retainFocus
            tabbable={false}
            className="flex-1"
            onClick={pickImage}
          />
        </div>
        <Text data-part="app.draftinspector.hint" as="p" size="sm" tone="muted" className="mt-3 leading-4">
          Type a title, add a note, or add an image — the task is created the moment it has something in it.
        </Text>
      </div>
    </div>
  );
}
