import { useRef, type KeyboardEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@taskscape/ui/button";
import { Text } from "@taskscape/ui/text";
import { toastVariants } from "@taskscape/ui/toast";
import { useSettings } from "../../api/queries";
import { dismissToast, failToast, useToastStore, type Restored, type UndoFn } from "./store";

export interface ToastHostProps {
  /** An Undo succeeded: the count of root tasks restored and the first one's title. */
  onUndone?: (restored: Restored) => void;
  /** The notice closed by Dismiss, or by a successful Undo. Not on a failed Undo or a timeout. */
  onDismissed?: (via: "dismiss" | "undo") => void;
}

/** The reduced-motion setting, resolved: On, or System with the OS preference set. */
function useReducedMotion(): boolean {
  const { data } = useSettings();
  const osPrefers = typeof window !== "undefined" && window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
  const setting = data?.reduced_motion;
  return setting === "on" || (setting !== "off" && !!osPrefers);
}

/** Enter and Space press the focused control explicitly, as the widget Toast does, so keys do not rely on the native click. */
function pressOnKey(event: KeyboardEvent<HTMLButtonElement>) {
  if (event.key === "Enter" || event.key === " " || event.key === "Space") {
    event.preventDefault();
    event.currentTarget.click();
  }
}

/** A native Space click fires on keyup; cancelling it keeps one Space press to one click. */
function cancelSpaceKeyUp(event: KeyboardEvent<HTMLButtonElement>) {
  if (event.key === " " || event.key === "Space") event.preventDefault();
}

/** The undo notice pinned bottom-centre of the window. It renders nothing when no notice is set. */
export function ToastHost({ onUndone, onDismissed }: ToastHostProps) {
  const notice = useToastStore((s) => s.notice);
  const qc = useQueryClient();
  const reduce = useReducedMotion();
  const busy = useRef(false);
  // The Undo runs as a mutation, so the page waits for it like any other write.
  const undoMutation = useMutation({ mutationFn: (undo: UndoFn) => undo() });

  if (!notice) return null;

  const onUndo = async () => {
    if (busy.current) return;
    busy.current = true;
    try {
      const restored = await undoMutation.mutateAsync(notice.undo);
      for (const k of ["lists", "tasks", "notes", "trash", "projects"]) qc.invalidateQueries({ queryKey: [k] });
      // The restore happened, so it is reported even if the timer or a newer notice closed this one meanwhile.
      onUndone?.(restored);
      if (dismissToast(notice.id)) onDismissed?.("undo");
    } catch {
      failToast(notice.id, "Could not undo the delete.");
    } finally {
      busy.current = false;
    }
  };

  const onDismiss = () => {
    if (dismissToast(notice.id)) onDismissed?.("dismiss");
  };

  return (
    <div className="pointer-events-none fixed inset-x-0 bottom-[1.214rem] z-(--z-index-popover) flex justify-center">
      <div
        key={notice.id}
        data-part="app.toasthost.notice"
        className={`${toastVariants({ tone: "neutral" })}${reduce ? "" : " animate-rise"}`}
      >
        <Text as="span" role="status" data-part="app.toasthost.message" size="md" weight="medium" overflow="truncate">
          {notice.failed ? notice.errorMessage : notice.message}
        </Text>
        <Button
          data-part="app.toasthost.undo"
          variant="ghost"
          tone="accent"
          size="md"
          width="content"
          icon="undo"
          iconSize={14}
          label={notice.failed ? "Try again" : "Undo"}
          onClick={onUndo}
          onKeyDown={pressOnKey}
          onKeyUp={cancelSpaceKeyUp}
        />
        <Button
          data-part="app.toasthost.dismiss"
          variant="ghost"
          tone="muted"
          size="md"
          shape="icon"
          width="content"
          icon="close"
          iconSize={14}
          ariaLabel="Dismiss"
          description="Dismiss"
          onClick={onDismiss}
          onKeyDown={pressOnKey}
          onKeyUp={cancelSpaceKeyUp}
        />
      </div>
    </div>
  );
}
