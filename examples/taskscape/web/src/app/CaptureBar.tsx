// Quick capture, the web's version of the tray's capture bar
// (tauri/tray/src/App.tsx, its components and its draft hook): a page has no
// menu bar to live in and can't be summoned over other apps, so it is a bar in
// the page, summoned anywhere in it with the capture shortcut (⌘↩ / Ctrl+↩) or
// the header's bolt. Like the tray's window it is kept alive between uses
// (hidden, not unmounted), so it is ready at once and the draft is still in
// it. It keeps the bar's rules: it files into the list in use (the focused
// list, else the first), Tab opens the notes (the rich-text editor), Enter
// (⌘↩ in the notes) files the task with the notes as its first note and
// empties the draft, and the draft survives Escape or a press outside; only
// filing it or Clear empties it. Pressing where the task will go shows that
// list. There are no screenshots in a browser.
import { cn } from '@taskscape/common-ui/cn';
import { Divider, HotkeyHint, ToolbarButton } from '@taskscape/common-ui/components';
import { Icon } from '@taskscape/common-ui/Icon';
import { formatAccel, matchesEvent, parseAccel } from '@taskscape/common-ui/hotkeys';
import { useDismissable } from '@taskscape/common-ui/overlay';
import { RichTextEditor, type RichTextHandle } from '@shared/components/preview/RichTextEditorLazy';
import { useHotkeyStore } from '@shared/stores/hotkeyStore';
import { useLayoutStore } from '@shared/stores/layoutStore';
import { useListStore } from '@shared/stores/listStore';
import { useProjectStore } from '@shared/stores/projectStore';
import { useTaskStore } from '@shared/stores/taskStore';
import { useEffect, useRef, useState, type RefObject } from 'react';
import { create } from 'zustand';
import { api } from './api';
import { go } from './url';
import { useWebUi } from './webUi';

/** The draft outlives the bar being shown. The notes' HTML lives in the
 *  editor, which stays mounted while the bar is hidden. */
const useDraft = create<{ title: string; notesOpen: boolean; hasNotes: boolean }>(() => ({
  title: '',
  notesOpen: false,
  hasNotes: false,
}));

function useTarget() {
  const lists = useListStore((s) => s.lists);
  const projects = useProjectStore((s) => s.projects);
  const focused = useLayoutStore((s) => s.focusedListId());
  const list = lists.find((l) => l.id === focused) ?? lists[0] ?? null;
  const project = projects.find((p) => p.id === list?.project_id) ?? null;
  return { list, project };
}

/** Escape and a press outside hide the bar; mounted only while it shows, so
 *  the page's own keys aren't held off while it is hidden. */
function WhileShown({ card, onHide }: { card: RefObject<HTMLDivElement | null>; onHide: () => void }) {
  useDismissable({ onDismiss: onHide, ref: card, capture: true });
  return null;
}

/** The capture shortcut (the catalog's global ⌘↩, Ctrl+↩ off a Mac) summons
 *  the bar from anywhere in the page, a field included, and hides it again.
 *  The bar answers it itself, wherever it is mounted. */
function useCaptureShortcut() {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const accel = useHotkeyStore.getState().map['toggle_capture_bar'] ?? '';
      if (!matchesEvent(accel, e)) return;
      e.preventDefault();
      const ui = useWebUi.getState();
      ui.setCaptureOpen(!ui.captureOpen);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);
}

/** `page`: at the top of the window, over it. `fill`: the card alone at the
 *  top of its box (how the omnicui registry mounts it, as the tray's window is
 *  the card). */
export function CaptureBar({ placement = 'page' }: { placement?: 'page' | 'fill' }) {
  const open = useWebUi((s) => s.captureOpen);
  const { title, notesOpen, hasNotes } = useDraft();
  const { list, project } = useTarget();
  const card = useRef<HTMLDivElement>(null);
  const titleRef = useRef<HTMLInputElement>(null);
  const editorRef = useRef<RichTextHandle>(null);
  const [filing, setFiling] = useState(false);
  const clearAccel = useHotkeyStore((s) => s.map['clear_draft'] ?? '');
  const hide = () => useWebUi.getState().setCaptureOpen(false);
  const hasDraft = title.length > 0 || hasNotes;
  useCaptureShortcut();

  useEffect(() => {
    if (open) titleRef.current?.focus();
  }, [open]);

  const clear = () => {
    editorRef.current?.clear?.();
    useDraft.setState({ title: '', notesOpen: false, hasNotes: false });
    titleRef.current?.focus();
  };

  const file = async () => {
    const name = useDraft.getState().title.trim();
    if (!name || !list || filing) return;
    setFiling(true);
    try {
      const notes = editorRef.current && !editorRef.current.isEmpty() ? editorRef.current.getHtml() : null;
      const task = await api.createTask(list.id, name);
      if (notes) await api.createNote(task.id, notes);
      await useTaskStore.getState().load();
      clear();
    } finally {
      setFiling(false);
    }
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    // The page's own shortcuts wait while the bar has the keyboard.
    e.stopPropagation();
    if (matchesEvent(clearAccel, e.nativeEvent)) {
      e.preventDefault();
      clear();
    }
  };

  return (
    <div
      hidden={!open}
      className={cn(
        'pointer-events-none flex justify-center',
        placement === 'page' ? 'z-overlay fixed inset-x-0 top-[12vh] px-4' : 'w-full'
      )}
    >
      {open && <WhileShown card={card} onHide={hide} />}
      <div
        ref={card}
        role="dialog"
        aria-label="Quick capture"
        data-part="captureBar"
        onKeyDown={onKeyDown}
        className={cn(
          'rounded-panel border-edge-2l dark:border-edge-2d bg-surface-2l dark:bg-surface-2d text-content-1l dark:text-content-1d pointer-events-auto flex w-[min(480px,100%)] flex-col overflow-hidden border',
          placement === 'page' && 'shadow-menu animate-sheet-in'
        )}
      >
        <div className="flex h-10 shrink-0 items-center px-3 py-2">
          <input
            ref={titleRef}
            data-part="captureTitle"
            value={title}
            aria-label="Capture a task ..."
            placeholder="Capture a task ..."
            spellCheck={false}
            onChange={(e) => useDraft.setState({ title: e.target.value })}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                void file();
              } else if (e.key === 'Tab' && !e.shiftKey && !notesOpen) {
                e.preventDefault();
                useDraft.setState({ notesOpen: true });
              }
            }}
            className="text-content-1l dark:text-content-1d placeholder:text-content-3l dark:placeholder:text-content-3d min-w-0 flex-1 bg-transparent text-sm outline-none"
          />
          {hasDraft && (
            <ToolbarButton
              data-part="captureClear"
              size="xs"
              variant="well"
              icon="delete_sweep"
              iconSize={16}
              onClick={clear}
              tabIndex={-1}
              className="text-content-2l hover:text-content-1l dark:text-content-2d dark:hover:text-content-1d shrink-0 gap-0 text-[11px]"
              title={clearAccel ? `Clear the draft (${formatAccel(clearAccel)})` : 'Clear the draft'}
            >
              <p className="px-1">Clear</p>
              <HotkeyHint hotkey={parseAccel(clearAccel)} />
            </ToolbarButton>
          )}
        </div>

        <Divider className="shrink-0" />

        {/* The notes open at once, as the tray's do (its window then fits the
            card); there is no transition to match. */}
        <div className="grid">
          <div className="min-h-0 overflow-hidden">
            {notesOpen && (
              <RichTextEditor
                ref={editorRef}
                part="captureNotes"
                autoFocus
                attachments={[]}
                placeholder="Notes ..."
                minHeightClass="min-h-16"
                maxHeightClass="max-h-40"
                floatingToolbar={false}
                wrapperClassName=""
                toolbarClassName="border-edge-1l dark:border-edge-1d gap-1 border-t px-2 py-1.5"
                submitOnModEnter
                onSubmit={() => void file()}
                onEscape={hide}
                onChange={() => useDraft.setState({ hasNotes: !editorRef.current?.isEmpty() })}
                // Emptied and left, the notes close again.
                onBlur={() => {
                  if (editorRef.current?.isEmpty()) useDraft.setState({ notesOpen: false, hasNotes: false });
                }}
              />
            )}
          </div>
        </div>
        {!notesOpen && (
          <button
            type="button"
            data-part="captureNotesHint"
            onClick={() => useDraft.setState({ notesOpen: true })}
            className="text-content-3l dark:text-content-3d self-start px-3 py-2 text-xs"
          >
            Press Tab to add a note
          </button>
        )}
        <Divider />

        <div className="flex shrink-0 items-center justify-between gap-2 px-3 py-2">
          <button
            type="button"
            data-part="captureTarget"
            tabIndex={-1}
            disabled={!list}
            onClick={() => {
              if (!list) return;
              go({ project: null, list: list.id, task: null });
              hide();
            }}
            title="Show this list"
            className="text-content-2l hover:text-content-1l dark:text-content-2d dark:hover:text-content-1d flex min-w-0 items-center gap-1.5 text-xs"
          >
            <Icon name="arrow_outward" size={14} />
            <span className="truncate">{project && list ? `${project.name} / ${list.name}` : 'No list yet'}</span>
          </button>
        </div>
      </div>
    </div>
  );
}
