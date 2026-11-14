import * as React from "react";
import { Button } from "@taskscape/ui/button";
import { CharacterCounter } from "@taskscape/ui/character-counter";
import { RichTextEditor, type RichTextMention } from "@taskscape/ui/rich-text-editor";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { useWrite } from "../../api/queries";
import type { Id, Task } from "../../api/types";
import type { Emit } from "./events";
import { plainText } from "./plain";
import { noteTagger, useDomParts } from "./parts";
import { SectionTitle } from "./SectionTitle";

const NOTE_LIMIT = 2000;
const P = "app.taskinspector.";


interface NoteBodyProps {
  part: string;
  counterPart: string;
  value: string;
  placeholder?: string;
  autoFocus?: boolean;
  mentions: RichTextMention[];
  onChange: (text: string) => void;
  onBlur: (text: string) => void;
  onOpenMention: (id: string) => void;
  onRequestLink: () => void;
  deleteButton?: React.ReactNode;
}

/** One rich-text note editor, with its floating toolbar (the widget's) and its character counter. */
function NoteBody({
  part,
  counterPart,
  value,
  placeholder = "",
  autoFocus = false,
  mentions,
  onChange,
  onBlur,
  onOpenMention,
  onRequestLink,
  deleteButton,
}: NoteBodyProps) {
  const ref = React.useRef<HTMLDivElement>(null);
  const tag = React.useMemo(() => noteTagger(part), [part]);
  useDomParts(ref, tag);
  // Tags again after every render of this editor, in case the editor's own DOM changed without a mutation.
  React.useEffect(() => {
    if (ref.current) tag(ref.current);
  });
  const [focused, setFocused] = React.useState(false);
  const [length, setLength] = React.useState(value.length);

  return (
    <div
      ref={ref}
      className="group/row relative flex min-w-0 flex-col"
      onFocus={() => setFocused(true)}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setFocused(false);
      }}
    >
      <RichTextEditor
        ariaLabel="Note"
        value={value}
        placeholder={placeholder}
        autoFocus={autoFocus}
        maxLength={NOTE_LIMIT}
        mentions={mentions}
        onChange={(text) => {
          setLength(text.length);
          onChange(text);
        }}
        onBlur={onBlur}
        onOpenMention={onOpenMention}
        onRequestLink={onRequestLink}
      />
      {deleteButton}
      {focused && (
        <CharacterCounter data-part={counterPart} count={length} limit={NOTE_LIMIT} className="absolute right-1 top-full mt-1" />
      )}
    </div>
  );
}

interface NoteProps {
  taskTitle: string;
  savedCount: number;
  mentions: RichTextMention[];
  emit: Emit;
  onOpenMention: (id: string) => void;
  onRequestLink: () => void;
}

/** A saved note: its own editor, autosave while typing, and a save or discard on blur. */
function SavedNote({
  note,
  index,
  taskTitle,
  savedCount,
  mentions,
  emit,
  onOpenMention,
  onRequestLink,
}: NoteProps & { note: { id: Id; content: string }; index: number }) {
  const updateNote = useWrite(ep.notes.update);
  const removeNote = useWrite(ep.notes.remove);
  const text0 = plainText(note.content);
  const saved = React.useRef(text0);

  const persist = (text: string) => {
    if (text === saved.current) return;
    saved.current = text;
    updateNote.mutate([note.id, { content: text }]);
    emit("noteSaved", { text });
  };

  // Autosave: each change is written as it is made (the contract reads the save straight after typing);
  // an emptied note waits for its blur.
  const onChange = (text: string) => {
    if (text !== "") persist(text);
  };

  const onBlur = (text: string) => {
    if (text === "") {
      // An emptied note is discarded at once, with no confirmation.
      if (saved.current !== "") {
        saved.current = "";
        removeNote.mutate([note.id]);
        emit("notesChanged", { count: savedCount - 1 });
      }
      return;
    }
    persist(text);
  };

  const first = index === 0;
  return (
    <NoteBody
      part={first ? `${P}noteBody` : `${P}noteBodySecond`}
      counterPart={first ? `${P}noteChars` : `${P}noteCharsSecond`}
      value={text0}
      mentions={mentions}
      onChange={onChange}
      onBlur={onBlur}
      onOpenMention={onOpenMention}
      onRequestLink={onRequestLink}
      deleteButton={
        <Button
          data-part={first ? `${P}noteDelete` : undefined}
          ariaLabel="Delete note"
          description="Delete note"
          icon="close"
          iconSize={14}
          shape="icon"
          size="sm"
          variant="filled"
          tone="muted"
          reveal="row"
          className="absolute -right-1.5 -top-1.5 z-10 bg-surface-3 shadow-menu"
          onClick={() => emit("deleteRequested", { target: "note", title: taskTitle })}
        />
      }
    />
  );
}

/** The add-note editor. Its note is created on the first pause in typing and then kept up to date. */
function DraftNote({
  taskId,
  savedCount,
  mentions,
  emit,
  onCreated,
  onFinished,
  onOpenMention,
  onRequestLink,
}: {
  taskId: Id;
  savedCount: number;
  mentions: RichTextMention[];
  emit: Emit;
  onCreated: (id: Id | null) => void;
  onFinished: () => void;
  onOpenMention: (id: string) => void;
  onRequestLink: () => void;
}) {
  const createNote = useWrite(ep.notes.create);
  const updateNote = useWrite(ep.notes.update);
  const removeNote = useWrite(ep.notes.remove);
  const draftId = React.useRef<Id | null>(null);
  const saved = React.useRef("");
  // Writes run one after another, so keystrokes and a blur cannot create the note twice.
  const queue = React.useRef<Promise<void>>(Promise.resolve());

  const write = (text: string) => {
    queue.current = queue.current
      .then(async () => {
        if (text === saved.current) return;
        if (draftId.current === null) {
          if (text === "") return;
          const note = await createNote.mutateAsync([taskId, text]);
          draftId.current = note.id;
          saved.current = text;
          onCreated(note.id);
          emit("notesChanged", { count: savedCount + 1 });
          emit("noteSaved", { text });
          return;
        }
        if (text === "") {
          const id = draftId.current;
          draftId.current = null;
          saved.current = "";
          onCreated(null);
          await removeNote.mutateAsync([id]);
          emit("notesChanged", { count: savedCount });
          return;
        }
        await updateNote.mutateAsync([draftId.current, { content: text }]);
        saved.current = text;
        emit("noteSaved", { text });
      })
      .catch((error: unknown) => {
        console.error("task-inspector: note write failed", error);
      });
    return queue.current;
  };

  return (
    <NoteBody
      part={`${P}newNoteEditor`}
      counterPart={`${P}noteChars`}
      value=""
      placeholder="Write a note ... (drop an attachment to @mention it)"
      autoFocus
      mentions={mentions}
      onChange={(text) => void write(text)}
      onBlur={(text) => {
        // Finishing waits for the window to have focus (F03-029): a blur from switching apps keeps the draft.
        const windowFocused = document.hasFocus();
        void write(text).then(() => {
          if (windowFocused) onFinished();
        });
      }}
      onOpenMention={onOpenMention}
      onRequestLink={onRequestLink}
    />
  );
}

/** The Notes section: the saved notes, the add-note editor, and the add control (header or dashed box). */
export function NotesSection({
  task,
  mentions,
  emit,
  onOpenMention,
  onRequestLink,
}: {
  task: Task;
  mentions: RichTextMention[];
  emit: Emit;
  onOpenMention: (id: string) => void;
  onRequestLink: () => void;
}) {
  const [adding, setAdding] = React.useState(false);
  const [draftId, setDraftId] = React.useState<Id | null>(null);
  const notes = task.note_items;
  // The stored HTML is read as its text (see plain.ts); the editor edits that text.
  const count = notes.length;
  // While its note is being written, the draft editor stands in for that note.
  const shown = notes.filter((note) => note.id !== draftId);

  return (
    <div className="flex min-w-0 flex-col gap-2">
      <SectionTitle
        label="Notes"
        part={`${P}notesHeading`}
        trailing={
          count > 0 ? (
            <>
              <Text data-part={`${P}notesCount`} size="xs" tone="muted" numeric="tabular">
                {String(count)}
              </Text>
              <Button
                data-part={`${P}addNote`}
                ariaLabel="Add note"
                icon="add"
                iconSize={14}
                label="Note"
                variant="ghost"
                size="sm"
                width="content"
                truncate={false}
                tone="muted"
                className="font-semibold"
                onClick={() => setAdding(true)}
              />
            </>
          ) : undefined
        }
      />
      <div className="flex min-w-0 flex-col gap-[0.429rem]">
        {shown.map((note, index) => (
          <SavedNote
            key={note.id}
            note={note}
            index={index}
            taskTitle={task.title}
            savedCount={count}
            mentions={mentions}
            emit={emit}
            onOpenMention={onOpenMention}
            onRequestLink={onRequestLink}
          />
        ))}
        {adding && (
          <DraftNote
            key="draft"
            taskId={task.id}
            savedCount={count}
            mentions={mentions}
            emit={emit}
            onCreated={setDraftId}
            onFinished={() => {
              setDraftId(null);
              setAdding(false);
            }}
            onOpenMention={onOpenMention}
            onRequestLink={onRequestLink}
          />
        )}
        {count === 0 && !adding && (
          <Button
            data-part={`${P}addNote`}
            variant="dashed"
            size="dashedTall"
            width="full"
            icon="add"
            iconSize={16}
            label="Add note"
            tone="muted"
            className="bg-surface-3 font-semibold"
            onClick={() => setAdding(true)}
          />
        )}
      </div>
    </div>
  );
}
