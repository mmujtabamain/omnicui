// One row of the task list: the round check, the title and note line, the attachment and subtask counts,
// the disclosure chevron and the More menu. Also the inline title field and the subtask composer.
import { useEffect, useRef, useState, type KeyboardEvent, type ReactNode, type RefObject } from "react";
import { Badge } from "@taskscape/ui/badge";
import { Button } from "@taskscape/ui/button";
import { Icon } from "@taskscape/ui/icon";
import { Input } from "@taskscape/ui/input";
import { MatchHighlight } from "@taskscape/ui/match-highlight";
import { Separator } from "@taskscape/ui/separator";
import type { Id } from "../../api/types";
import { cx, plainNotes, type RowModel } from "./model";

const IS_MAC = typeof navigator !== "undefined" && /Mac|iPhone|iPad|iPod/.test(navigator.platform);

/** Contract parts a row can carry (the full names are app.tasklist.<part>). */
export interface RowParts {
  row?: string;
  check?: string;
  more?: string;
  disclosure?: string;
  badge?: string;
  attach?: string;
  subtask?: string;
  mark?: string;
}

export interface RowProps {
  row: RowModel;
  /** The query as typed, for the highlight. */
  query: string;
  searching: boolean;
  picked: boolean;
  previewed: boolean;
  /** Option (Alt) is held: the check is a square that marks done. */
  optionHeld: boolean;
  parts: RowParts;
  renaming: boolean;
  /** The More menu, which carries the row's trigger. */
  menu: ReactNode;
  /** A press on the row body: modifier is Control (Command on macOS), shift extends the range. */
  onActivate: (row: RowModel, modifier: boolean, shift: boolean) => void;
  onCheck: (row: RowModel, alt: boolean) => void;
  /** A double-click on the row body toggles done. */
  onDoubleToggle: (row: RowModel) => void;
  /** Right-click opens the row's menu. */
  onContext: (row: RowModel) => void;
  onToggleExpanded: (row: RowModel) => void;
  onRenamed: (id: Id, title: string | null) => void;
  onKey: (event: KeyboardEvent<HTMLDivElement>, row: RowModel) => void;
  registerBody: (id: Id, el: HTMLElement | null) => void;
}

const P = (part: string) => `app.tasklist.${part}`;

/** Left padding by nesting level: the root check sits at 20px, each level adds 24px (1rem = 14px). */
const INDENT = ["pl-[1.429rem]", "pl-[3.143rem]", "pl-[4.857rem]", "pl-[6.571rem]", "pl-[8.286rem]"];
export const indentClass = (depth: number): string => INDENT[Math.min(Math.max(depth, 0), INDENT.length - 1)] ?? "";

/** Focuses a field now and again shortly after: a menu closing hands focus back to its trigger after a moment. */
function useFocusSoon(field: RefObject<HTMLInputElement | HTMLTextAreaElement | null>) {
  useEffect(() => {
    const timers = [0, 40, 160].map((ms) =>
      window.setTimeout(() => {
        const el = field.current;
        if (el && document.activeElement !== el) el.focus();
      }, ms),
    );
    return () => timers.forEach((t) => window.clearTimeout(t));
  }, [field]);
}

/** Text with the query's matches filled; the first fill takes the part name when mark is set. */
function Marked({ text, query, mark }: { text: string; query: string; mark?: string }) {
  const q = query.trim().toLowerCase();
  if (q === "") return <>{text}</>;
  const lower = text.toLowerCase();
  const out: ReactNode[] = [];
  let at = 0;
  let hit = lower.indexOf(q);
  let marks = 0;
  while (hit >= 0) {
    if (hit > at) out.push(text.slice(at, hit));
    const part = marks === 0 && mark ? { "data-part": mark } : {};
    out.push(<MatchHighlight key={hit} query={q} text={text.slice(hit, hit + q.length)} {...part} />);
    marks += 1;
    at = hit + q.length;
    hit = lower.indexOf(q, at);
  }
  if (at < text.length) out.push(text.slice(at));
  return <>{out}</>;
}

export function TaskRow(p: RowProps) {
  const row = p.row;
  const t = row.task;
  const title = t.title;
  const note = plainNotes(t.notes);
  const q = p.query.trim().toLowerCase();
  const titleHit = q !== "" && title.toLowerCase().includes(q);
  const noteHit = q !== "" && note.toLowerCase().includes(q);
  const tinted = p.picked || p.previewed;
  const hasKids = row.kids > 0 && !p.searching;
  const attachments = t.attachments.length;
  // Option held, or a done task: the name says what the check does now.
  const checkName =
    p.optionHeld || t.done ? (t.done ? "Mark not done" : "Mark done") : p.picked ? "Deselect" : "Select";

  const fill = p.optionHeld
    ? t.done
      ? "border-done-lamp-1 bg-done-lamp-1 text-on-accent"
      : "border-content-1 bg-surface-3"
    : p.picked
      ? "border-accent-500 bg-accent-500 text-on-accent"
      : "border-content-1 bg-surface-3";
  const glyph = p.optionHeld
    ? t.done
      ? <Icon name="check" size={14} weight={700} tone="inherit" />
      : null
    : p.picked
      ? <span aria-hidden="true" className="block size-[0.429rem] rounded-full bg-on-accent" />
      : null;

  return (
    <div
      data-task-id={t.id}
      data-picked={p.picked || undefined}
      data-part={p.parts.subtask}
      onKeyDown={(e) => p.onKey(e, row)}
      onContextMenu={(e) => {
        e.preventDefault();
        p.onContext(row);
      }}
      className={cx(
        "group/row relative flex min-h-[2.286rem] items-center gap-[0.643rem] rounded-(--radius-field) pr-[0.714rem] text-content-1",
        // A plain row is 32px tall; a row with a note line takes the note's 46px, as in the reference.
        note !== "" ? "py-[0.429rem]" : "py-0",
        indentClass(row.depth),
        tinted ? "bg-selection-1" : "hover:bg-wash-1",
      )}
    >
      <span
        role="button"
        tabIndex={0}
        aria-label={checkName}
        aria-selected={p.picked || undefined}
        data-slot="check"
        data-part={p.parts.check}
        onClick={(e) => p.onCheck(row, e.altKey)}
        // A double-click on the check does not toggle done.
        onDoubleClick={(e) => e.stopPropagation()}
        className="flex shrink-0 cursor-default rounded-full outline-none focus-visible:ring-2 focus-visible:ring-focus-1"
      >
        <span
          className={cx(
            "flex size-[1.286rem] items-center justify-center border",
            p.optionHeld ? "rounded-(--radius-field)" : "rounded-full",
            fill,
          )}
        >
          {glyph}
        </span>
      </span>

      <div className="flex min-w-0 flex-1 flex-col">
        {p.renaming ? (
          <RenameField initial={title} onDone={(text) => p.onRenamed(t.id, text)} />
        ) : (
          <span
            role="button"
            tabIndex={0}
            aria-label={title}
            aria-selected={tinted || undefined}
            data-part={p.parts.row}
            ref={(el) => p.registerBody(t.id, el)}
            onMouseDown={(e) => {
              // A shift-press extends the selection; it must not start a native text selection.
              if (e.shiftKey) e.preventDefault();
            }}
            onClick={(e) => p.onActivate(row, IS_MAC ? e.metaKey : e.ctrlKey, e.shiftKey)}
            onDoubleClick={() => p.onDoubleToggle(row)}
            className="flex min-w-0 cursor-default flex-col rounded-(--radius-field) outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1"
          >
            <span
              className={cx(
                "block truncate text-[1rem] leading-[1.357rem] font-medium",
                t.done ? "text-content-3 line-through" : "text-content-1",
              )}
            >
              <Marked text={title} query={p.query} mark={titleHit ? p.parts.mark : undefined} />
            </span>
            {note !== "" && (
              <span
                className={cx(
                  "block truncate text-[0.893rem] leading-[1.1rem] text-content-2",
                  t.done && "opacity-55",
                )}
              >
                <Marked text={note} query={p.query} mark={!titleHit && noteHit ? p.parts.mark : undefined} />
              </span>
            )}
          </span>
        )}
      </div>

      {attachments > 0 && (
        <Badge data-part={p.parts.attach} tone="outline" size="md" icon>
          {String(attachments)}
        </Badge>
      )}
      {hasKids && (
        <Button
          data-part={p.parts.disclosure}
          data-row-control
          variant="ghost"
          tone="muted"
          size="sm"
          shape="icon"
          icon="chevron_left"
          iconSize={16}
          ariaLabel={row.expanded ? "Collapse" : "Expand"}
          reveal={row.expanded ? "row" : "always"}
          className={cx("shrink-0", row.expanded && "-rotate-90")}
          onClick={() => p.onToggleExpanded(row)}
        />
      )}
      {hasKids && (
        <Badge data-part={p.parts.badge} tone="outline" size="md">
          {`${row.kidsDone}/${row.kids}`}
        </Badge>
      )}
      {p.menu}
      <Separator inset="row" depth={row.depth} />
    </div>
  );
}

/** The inline title field of a row being renamed: the whole title is selected when it opens. */
export function RenameField({ initial, onDone }: { initial: string; onDone: (text: string | null) => void }) {
  const [text, setText] = useState(initial);
  const field = useRef<HTMLInputElement | HTMLTextAreaElement | null>(null);
  const finished = useRef(false);
  // Focus can pass to a closing menu's trigger for a moment; blur does not end the edit then.
  const settleAt = useRef(0);
  useEffect(() => {
    settleAt.current = Date.now() + 300;
  }, []);
  useFocusSoon(field);

  const finish = (value: string | null) => {
    if (finished.current) return;
    finished.current = true;
    onDone(value);
  };

  return (
    <div
      className="flex min-w-0 flex-1"
      onBlur={(e) => {
        if (e.relatedTarget instanceof Node && e.currentTarget.contains(e.relatedTarget)) return;
        if (Date.now() < settleAt.current) return;
        finish(text.trim());
      }}
    >
      <Input
        data-part={P("renameField")}
        ref={field}
        mode="bare"
        size="md"
        ariaLabel="Task title"
        spellCheck={false}
        value={text}
        autoFocus
        selectOnFocus
        stopKeyPropagation
        className="min-w-0 flex-1 focus:ring-0"
        onValueChange={setText}
        onSubmit={(value) => finish(value)}
        onCancel={() => finish(null)}
      />
    </div>
  );
}

/** The inline field under a parent row: Enter adds a subtask, an empty Enter or Escape or blur dismisses it. */
export function SubtaskComposer({
  depth,
  onSubmit,
  onDismiss,
}: {
  depth: number;
  onSubmit: (title: string) => void;
  onDismiss: () => void;
}) {
  const [text, setText] = useState("");
  const field = useRef<HTMLInputElement | HTMLTextAreaElement | null>(null);
  const settleAt = useRef(0);
  useEffect(() => {
    settleAt.current = Date.now() + 300;
  }, []);
  useFocusSoon(field);

  return (
    <div
      className={cx("flex h-[2.571rem] w-full min-w-0 items-center gap-[0.429rem] pr-2", indentClass(depth))}
      onBlur={(e) => {
        if (e.relatedTarget instanceof Node && e.currentTarget.contains(e.relatedTarget)) return;
        if (Date.now() < settleAt.current) return;
        onDismiss();
      }}
    >
      <span className="flex shrink-0 items-center text-content-3">
        <Icon name="subdirectory_arrow_right" size={15} weight={300} tone="inherit" />
      </span>
      <Input
        data-part={P("subtaskComposer")}
        ref={field}
        mode="bare"
        size="md"
        ariaLabel="Subtask title"
        placeholder="Subtask title — Enter to add"
        spellCheck={false}
        value={text}
        autoFocus
        stopKeyPropagation
        className="min-w-0 flex-1 focus:ring-0"
        onValueChange={setText}
        onSubmit={(value) => {
          if (value === "") {
            onDismiss();
            return;
          }
          setText("");
          onSubmit(value);
        }}
        onCancel={onDismiss}
      />
    </div>
  );
}
