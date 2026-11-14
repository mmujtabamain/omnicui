// SheetHost: the one sheet that is up, as a card over a soft scrim (web: in-page dialog), and the
// "Moved to Trash" toast. Accept performs the write itself, then resolves the sheet; a failed
// write keeps the sheet open with an inline line.
import * as React from "react";
import { Button } from "@taskscape/ui/button";
import { DialogButton, dialogButtonVariants } from "@taskscape/ui/dialog-button";
import { Footer } from "@taskscape/ui/footer";
import { Icon } from "@taskscape/ui/icon";
import { Input } from "@taskscape/ui/input";
import { Overlay, OverlayTitle } from "@taskscape/ui/overlay";
import { Scrim } from "@taskscape/ui/scrim";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { useLists, useWrite } from "../../api/queries";
import type { TaskList } from "../../api/types";
import { rollListName, rollProjectName } from "./names";
import { type SheetResult, type SheetSpec, type ToastState, useSheets } from "./store";

type Commit = (spec: SheetSpec, name: string) => Promise<SheetResult>;

/** The writes behind each sheet's accept. Hooks live here so they outlive the card. */
function useCommit(): Commit {
  const createList = useWrite(ep.lists.create);
  const createProject = useWrite(ep.projects.create);
  const renameList = useWrite(ep.lists.update);
  const renameProject = useWrite(ep.projects.update);
  const removeList = useWrite(ep.lists.remove);
  const removeProject = useWrite(ep.projects.remove);
  const trash = useWrite(ep.tasks.trash);
  const restore = useWrite(ep.trash.restore);

  return async (spec, name) => {
    switch (spec.kind) {
      case "newList":
        await createList.mutateAsync([spec.projectId, name]);
        return { action: "accept", text: name };
      case "newProject":
        await createProject.mutateAsync([name]);
        return { action: "accept", text: name };
      case "renameList":
        await renameList.mutateAsync([spec.list.id, { name }]);
        return { action: "accept", text: name };
      case "renameProject":
        await renameProject.mutateAsync([spec.project.id, { name }]);
        return { action: "accept", text: name };
      case "deleteList":
        await removeList.mutateAsync([spec.list.id]);
        return { action: "accept", text: "" };
      case "deleteProject":
        await removeProject.mutateAsync([spec.project.id]);
        return { action: "accept", text: "" };
      case "deleteConfirm": {
        const res = await trash.mutateAsync([spec.task.id]);
        const ids = res?.trashed?.length ? res.trashed : [spec.task.id];
        useSheets.getState().setToast({
          message: "Moved to Trash",
          failed: false,
          errorMessage: "Could not undo",
          retry: async () => {
            await restore.mutateAsync([ids]);
          },
        });
        return { action: "accept", text: "" };
      }
    }
  };
}

/** The toast's action: Undo, or Try again after a failure. A failure brings the toast back with the error. */
function runToastAction(toast: ToastState) {
  useSheets.getState().setToast(null);
  toast.retry?.().catch(() => useSheets.getState().setToast({ ...toast, failed: true }));
}

const TOAST_MS = 6000;

/**
 * The "Moved to Trash" toast, bottom centre. Always mounted and hidden while there is none, so its
 * parts exist in the page. Each new toast restarts the 6-second timer.
 */
function TrashToast() {
  const toast = useSheets((s) => s.toast);

  React.useEffect(() => {
    if (!toast) return undefined;
    const timer = window.setTimeout(() => useSheets.getState().setToast(null), TOAST_MS);
    return () => window.clearTimeout(timer);
  }, [toast]);

  return (
    <div className="pointer-events-none fixed inset-x-0 bottom-[1.429rem] z-(--z-index-popover) flex justify-center">
      <div
        data-part="app.sheets.toast"
        role="status"
        className={`pointer-events-auto relative flex h-[2.929rem] max-w-[calc(100%-2.286rem)] items-center gap-2 rounded-(--radius-control) border border-edge-2 bg-surface-3 pr-1.5 pl-3.5 shadow-lift animate-rise ${toast ? "" : "hidden"}`}
      >
        <Text size="md" overflow="truncate">
          {toast ? (toast.failed ? toast.errorMessage : toast.message) : ""}
        </Text>
        <Button
          data-part="app.sheets.toastUndo"
          variant="ghost"
          tone="strong"
          hover="wash"
          size="sm"
          width="content"
          label={toast?.failed ? "Try again" : "Undo"}
          onClick={() => toast && runToastAction(toast)}
        />
        <Button
          data-part="app.sheets.toastClose"
          variant="ghost"
          tone="muted"
          hover="wash"
          size="sm"
          shape="icon"
          width="content"
          icon="close"
          iconSize={16}
          ariaLabel="Dismiss"
          onClick={() => useSheets.getState().setToast(null)}
        />
      </div>
    </div>
  );
}

export function SheetHost() {
  const pending = useSheets((s) => s.pending);
  const commit = useCommit();

  return (
    <>
      {pending && <SheetCard key={pending.key} spec={pending.spec} commit={commit} />}
      <TrashToast />
    </>
  );
}

interface Prompt {
  initial: () => string;
  placeholder: string;
  roll: "list" | "project" | null;
}

interface Config {
  glyph: string;
  danger: boolean;
  title: string;
  detail: string;
  accept: string;
  instead: boolean;
  prompt: Prompt | null;
}

function count(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

function configFor(spec: SheetSpec, projectLists: TaskList[] | undefined): Config {
  switch (spec.kind) {
    case "newList":
      return {
        glyph: "list_alt_add", danger: false, title: "New list", detail: "", accept: "Create", instead: true,
        prompt: { initial: rollListName, placeholder: "Name this list", roll: "list" },
      };
    case "newProject":
      return {
        glyph: "create_new_folder", danger: false, title: "New project", detail: "", accept: "Create", instead: false,
        prompt: { initial: rollProjectName, placeholder: "Name this project", roll: "project" },
      };
    case "renameList":
      return {
        glyph: "edit", danger: false, title: "Rename list", detail: "", accept: "Rename", instead: false,
        prompt: { initial: () => spec.list.name, placeholder: "", roll: "list" },
      };
    case "renameProject":
      return {
        glyph: "edit", danger: false, title: "Rename project", detail: "", accept: "Rename", instead: false,
        prompt: { initial: () => spec.project.name, placeholder: "", roll: null },
      };
    case "deleteList": {
      const n = spec.list.task_count;
      return {
        glyph: "delete", danger: true, title: `Delete “${spec.list.name}”?`,
        detail: `Its ${count(n, "task", "tasks")} ${n === 1 ? "goes" : "go"} with it. No undo.`,
        accept: "Delete list", instead: false, prompt: null,
      };
    }
    case "deleteProject": {
      const lists = projectLists ?? [];
      const tasks = lists.reduce((sum, l) => sum + l.task_count, 0);
      return {
        glyph: "delete", danger: true, title: `Delete “${spec.project.name}”?`,
        detail: projectLists
          ? `Its ${count(lists.length, "list", "lists")} and ${count(tasks, "task", "tasks")} go with it. No undo.`
          : "",
        accept: "Delete project", instead: false, prompt: null,
      };
    }
    case "deleteConfirm":
      return {
        glyph: "delete", danger: true, title: "Move to Trash?",
        detail: `“${spec.task.title}” can be restored from Trash.`,
        accept: "Move to Trash", instead: false, prompt: null,
      };
  }
}

/** Footer buttons: 6rem (84px) on name sheets; 7.143rem (100px) on the deletion sheets, as the reference. */
function footerWidth(danger: boolean): string {
  return danger ? "min-w-[7.143rem]" : dialogButtonVariants();
}

function SheetCard({ spec, commit }: { spec: SheetSpec; commit: Commit }) {
  const projectLists = useLists(spec.kind === "deleteProject" ? spec.project.id : null);
  const cfg = configFor(spec, projectLists.data);
  const [text, setText] = React.useState(() => cfg.prompt?.initial() ?? "");
  const [busy, setBusy] = React.useState(false);
  const [error, setError] = React.useState("");
  const inputRef = React.useRef<HTMLInputElement | HTMLTextAreaElement>(null);
  const detailId = React.useId();
  const name = text.trim();

  const answer = (result: SheetResult) => useSheets.getState().close(result);

  const roll = () => {
    if (!cfg.prompt?.roll) return;
    setText(cfg.prompt.roll === "project" ? rollProjectName() : rollListName());
    requestAnimationFrame(() => {
      inputRef.current?.focus();
      inputRef.current?.select();
    });
  };

  // Accept: prompts need a non-empty name (Enter on an empty one refocuses the field).
  const submit = async (value: string) => {
    if (busy) return;
    if (cfg.prompt && value === "") {
      inputRef.current?.focus();
      return;
    }
    setBusy(true);
    setError("");
    try {
      answer(await commit(spec, value));
    } catch {
      setBusy(false);
      setError("Could not save that. Try again.");
    }
  };

  const danger = cfg.danger;

  return (
    <>
      <Scrim data-part="app.sheets.scrim" dim="soft" className="animate-[fade-in_var(--duration-fast)_var(--ease-standard)_both]" />
      <Overlay
        data-part="app.sheets.card"
        open
        onOpenChange={(open) => {
          if (!open) answer({ action: "dismiss" });
        }}
        aria-describedby={cfg.detail !== "" ? detailId : undefined}
        className="w-[min(26rem,100%)] overflow-hidden"
      >
        <div className="flex min-w-0 flex-col pt-[0.571rem] pr-[0.714rem] pb-[0.714rem] pl-[0.714rem]">
          <div className="flex min-w-0 items-center gap-[0.357rem]">
            <Icon name={cfg.glyph} tint={danger ? "danger" : "accent"} data-part="app.sheets.glyph" />
            <OverlayTitle data-part="app.sheets.headline" className="min-w-0 flex-1 font-display text-pretty">
              {cfg.title}
            </OverlayTitle>
          </div>
          {cfg.detail !== "" && (
            <Text
              as="p"
              id={detailId}
              data-part="app.sheets.detail"
              size="caption"
              tone="secondary"
              className="mt-[0.429rem]"
            >
              {cfg.detail}
            </Text>
          )}
          {cfg.prompt && (
            <div className="mt-[0.714rem] flex h-[2.5rem] min-w-0 items-center gap-[0.357rem] rounded-(--radius-field) border border-edge-2 bg-surface-1 pr-[0.429rem] pl-[0.714rem] focus-within:border-accent-500">
              <Input
                ref={inputRef}
                data-part="app.sheets.nameField"
                ariaLabel={cfg.title}
                mode="bare"
                size="md"
                selectOnFocus
                spellCheck={false}
                placeholder={cfg.prompt.placeholder}
                value={text}
                onValueChange={setText}
                onSubmit={(value) => void submit(value)}
                className="min-w-0 flex-1 focus:ring-0"
              />
              {cfg.prompt.roll && (
                <Button
                  data-part="app.sheets.roll"
                  variant="ghost"
                  size="md"
                  shape="icon"
                  tone="muted"
                  icon="casino"
                  iconSize={16}
                  ariaLabel="Roll another name"
                  tabbable={false}
                  retainFocus
                  onClick={roll}
                />
              )}
            </div>
          )}
          {error !== "" && (
            <Text as="p" size="caption" tone="danger" className="mt-[0.429rem]">
              {error}
            </Text>
          )}
        </div>
        <Footer density="row" edge="none" className="min-w-0 gap-[0.357rem] border-t border-edge-1 px-[0.714rem] py-[0.643rem]">
          {cfg.instead && (
            <DialogButton
              role="instead"
              insteadId="import"
              data-part="app.sheets.importJson"
              label="Import JSON…"
              className="min-w-[9.071rem]"
              onResolve={(id) => answer({ action: "instead", id })}
            />
          )}
          <div className="ml-auto grid grid-cols-2 gap-[0.357rem]">
            <Button
              data-part="app.sheets.cancel"
              variant="neutral"
              size="lg"
              width="content"
              label="Cancel"
              className={footerWidth(danger)}
              onClick={() => answer({ action: "dismiss" })}
            />
            <Button
              data-part="app.sheets.accept"
              variant={danger ? "danger" : "neutral"}
              size="lg"
              width="content"
              label={cfg.accept}
              className={footerWidth(danger)}
              disabled={cfg.prompt !== null && name === ""}
              busy={busy}
              onClick={() => void submit(name)}
              onKeyDown={(event) => {
                // Destructive confirmations: Enter does not confirm (decision 10). Space or a click does.
                if (danger && event.key === "Enter") event.preventDefault();
              }}
            />
          </div>
        </Footer>
      </Overlay>
    </>
  );
}
