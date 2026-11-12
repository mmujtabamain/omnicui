import { useLayoutEffect, useRef } from "react";
import { useStore, type TodoStore } from "../model.ts";
import { Parts } from "../omnicui/detail.ts";
import { CloseIcon } from "./icons.tsx";
import { Switch } from "./Switch.tsx";

/**
 * One task's detail, the pane right of the list: the title and the notes, edited in place (every
 * change is reported), the Done switch, Delete, and Close. `onLeave` is called when the user
 * leaves it: with Close (reported as closed) or by deleting the task.
 */
export function TaskDetailPane({ store, id, onLeave }: { store: TodoStore; id: string; onLeave?: () => void }) {
  useStore(store);
  const task = store.task(id);
  // Focus from the keyboard or from code puts the caret at the end of the text, as on every
  // platform (SPEC-APP.md, todo.detail). A field keeps its selection while it isn't focused and
  // Chromium restores it on focus, so the caret is put at the end once, when the task is shown;
  // a click still puts it where the click is.
  const titleRef = useRef<HTMLInputElement>(null);
  const notesRef = useRef<HTMLTextAreaElement>(null);
  useLayoutEffect(() => {
    for (const f of [titleRef.current, notesRef.current]) f?.setSelectionRange(f.value.length, f.value.length);
  }, [id]);
  if (!task) return <section className="pane pane--empty"><p>This task no longer exists.</p></section>;
  return (
    <section className="pane" aria-label="Task">
      <div className="pane__head">
        <h2 className="pane__title">Task</h2>
        <button type="button" className="icon-button" aria-label="Close" data-part={Parts.detailBack}
          onClick={() => { store.report("closed", { id }); onLeave?.(); }}>
          <CloseIcon />
        </button>
      </div>
      <label className="field">
        <span className="field__label">Title</span>
        <input ref={titleRef} className="field__input" value={task.title} onChange={(e) => store.rename(id, e.target.value)} data-part={Parts.title} />
      </label>
      <label className="field">
        <span className="field__label">Notes</span>
        <textarea ref={notesRef} className="field__input field__input--notes" rows={4} value={task.notes} onChange={(e) => store.editNotes(id, e.target.value)} data-part={Parts.notes} />
      </label>
      <Switch label="Done" checked={task.done} onChange={(done) => store.setDone(id, done)} part={Parts.done} />
      <div className="pane__actions">
        <button type="button" className="outlined-button outlined-button--danger" data-part={Parts.delete}
          onClick={() => { store.delete(id); onLeave?.(); }}>
          Delete
        </button>
      </div>
    </section>
  );
}

/** The detail pane when no task is open. */
export function NoTask() {
  return <section className="pane pane--empty"><p>Choose a task to see its details.</p></section>;
}
