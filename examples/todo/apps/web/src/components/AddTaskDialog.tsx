import { useLayoutEffect, useRef, useState } from "react";
import { Parts } from "../omnicui/list.ts";

/**
 * "New task", a modal <dialog>: a title field, Add (enabled once the title isn't blank) and
 * Cancel. Enter in the field adds too; Escape cancels. It grows in from 90% and fades in over
 * 200 ms, easing out (todo.css), as on every platform.
 */
export function AddTaskDialog({ onAdd, onClose }: { onAdd: (title: string) => void; onClose: () => void }) {
  const ref = useRef<HTMLDialogElement>(null);
  const [title, setTitle] = useState("");
  const canAdd = title.trim() !== "";
  useLayoutEffect(() => {
    const dialog = ref.current!;
    dialog.showModal();
    return () => dialog.close();
  }, []);
  return (
    <dialog ref={ref} className="dialog" aria-labelledby="new-task-title" data-part={Parts.newTaskDialog} onCancel={(e) => { e.preventDefault(); onClose(); }}>
      <form method="dialog" onSubmit={(e) => { e.preventDefault(); if (canAdd) onAdd(title); }}>
        <h2 id="new-task-title" className="dialog__title">New task</h2>
        <label className="field">
          <span className="field__label">Title</span>
          <input className="field__input" autoFocus value={title} onChange={(e) => setTitle(e.target.value)} data-part={Parts.newTitle} />
        </label>
        <div className="dialog__actions">
          <button type="button" className="text-button" onClick={onClose} data-part={Parts.cancelAdd}>Cancel</button>
          <button type="submit" className="text-button" disabled={!canAdd} data-part={Parts.confirmAdd}>Add</button>
        </div>
      </form>
    </dialog>
  );
}
