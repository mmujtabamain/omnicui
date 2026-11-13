// Quick capture, the web's version of the tray's capture bar
// (tauri/tray/src/App.tsx): a page has no menu bar to live in and can't be
// summoned over other apps, so it is a bar in the page, summoned anywhere in
// it with the capture shortcut (⌘↩ / Ctrl+↩) or the header's bolt. It keeps
// the bar's rules: it files into the list in use (the focused list, else the
// first), Tab opens the notes, Enter (⌘↩ in the notes) files the task with the
// notes as its first note, and the draft survives Escape or a click outside;
// only filing it or Clear empties it. There are no screenshots in a browser.
import { cn } from '@taskscape/common-ui/cn';
import { Icon } from '@taskscape/common-ui/Icon';
import { matchesEvent } from '@taskscape/common-ui/hotkeys';
import { useDismissable } from '@taskscape/common-ui/overlay';
import { useHotkeyStore } from '@shared/stores/hotkeyStore';
import { useLayoutStore } from '@shared/stores/layoutStore';
import { useListStore } from '@shared/stores/listStore';
import { useProjectStore } from '@shared/stores/projectStore';
import { useTaskStore } from '@shared/stores/taskStore';
import { useToastStore } from '@shared/stores/toastStore';
import { useEffect, useRef, useState } from 'react';
import { create } from 'zustand';
import { api } from './api';
import { useWebUi } from './webUi';

/** The draft outlives the bar being shown. */
const useDraft = create<{ title: string; notes: string; notesOpen: boolean }>(() => ({
  title: '',
  notes: '',
  notesOpen: false,
}));

const escapeHtml = (s: string) =>
  s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

/** Plain notes as the note editor's HTML: a paragraph per line. */
const notesHtml = (text: string) =>
  text
    .split('\n')
    .map((l) => `<p>${escapeHtml(l)}</p>`)
    .join('');

function useTarget() {
  const lists = useListStore((s) => s.lists);
  const projects = useProjectStore((s) => s.projects);
  const focused = useLayoutStore((s) => s.focusedListId());
  const list = lists.find((l) => l.id === focused) ?? lists[0] ?? null;
  const project = projects.find((p) => p.id === list?.project_id) ?? null;
  return { list, project };
}

export function CaptureBar() {
  const { title, notes, notesOpen } = useDraft();
  const { list, project } = useTarget();
  const close = () => useWebUi.getState().setCaptureOpen(false);
  const card = useRef<HTMLDivElement>(null);
  const titleRef = useRef<HTMLInputElement>(null);
  const notesRef = useRef<HTMLTextAreaElement>(null);
  const [filing, setFiling] = useState(false);
  const clearAccel = useHotkeyStore((s) => s.map['clear_draft'] ?? '');
  useDismissable({ onDismiss: close, ref: card, capture: true });

  useEffect(() => titleRef.current?.focus(), []);
  useEffect(() => {
    if (notesOpen) notesRef.current?.focus();
  }, [notesOpen]);

  const clear = () => {
    useDraft.setState({ title: '', notes: '', notesOpen: false });
    titleRef.current?.focus();
  };

  const file = async () => {
    const name = title.trim();
    if (!name || !list || filing) return;
    setFiling(true);
    try {
      const task = await api.createTask(list.id, name);
      if (notes.trim()) await api.createNote(task.id, notesHtml(notes.trim()));
      await useTaskStore.getState().load();
      useDraft.setState({ title: '', notes: '', notesOpen: false });
      close();
      useToastStore.getState().show(`Added to ${list.name}`);
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
    <div className="z-overlay pointer-events-none fixed inset-x-0 top-[12vh] flex justify-center px-4">
      <div
        ref={card}
        role="dialog"
        aria-label="Quick capture"
        data-part="captureBar"
        onKeyDown={onKeyDown}
        className="animate-sheet-in rounded-panel border-edge-2l dark:border-edge-2d bg-surface-2l dark:bg-surface-2d shadow-menu pointer-events-auto w-[min(480px,100%)] overflow-hidden border"
      >
        <div className="flex h-10 items-center gap-2 px-3">
          <Icon name="bolt" size={18} className="text-accent-500l dark:text-accent-500d shrink-0" />
          <input
            ref={titleRef}
            data-part="captureTitle"
            value={title}
            aria-label="Capture a task ..."
            placeholder="Capture a task ..."
            spellCheck={false}
            onChange={(e) => useDraft.setState({ title: e.target.value })}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                void file();
              } else if (e.key === 'Tab' && !e.shiftKey && !notesOpen) {
                e.preventDefault();
                useDraft.setState({ notesOpen: true });
              }
            }}
            className="text-content-1l dark:text-content-1d placeholder:text-content-3l dark:placeholder:text-content-3d min-w-0 flex-1 bg-transparent text-[15px] font-medium outline-none"
          />
          {(title || notes) && (
            <button
              type="button"
              data-part="captureClear"
              onClick={clear}
              className="rounded-field text-content-3l dark:text-content-3d hover:bg-wash-2l dark:hover:bg-wash-2d hover:text-content-1l dark:hover:text-content-1d px-2 py-0.5 text-[12px] font-medium"
            >
              Clear
            </button>
          )}
        </div>
        <div className={cn('grid transition-[grid-template-rows] duration-150', notesOpen ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]')}>
          <div className="overflow-hidden">
            <textarea
              ref={notesRef}
              data-part="captureNotes"
              value={notes}
              rows={3}
              aria-label="Notes"
              placeholder="Notes ..."
              tabIndex={notesOpen ? 0 : -1}
              onChange={(e) => useDraft.setState({ notes: e.target.value })}
              onBlur={() => {
                if (!notes.trim()) useDraft.setState({ notesOpen: false, notes: '' });
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
                  e.preventDefault();
                  void file();
                }
              }}
              className="text-content-1l dark:text-content-1d placeholder:text-content-3l dark:placeholder:text-content-3d block w-full resize-none bg-transparent px-10 pb-2 text-[13px] leading-5 outline-none"
            />
          </div>
        </div>
        <div className="border-edge-1l dark:border-edge-1d text-content-3l dark:text-content-3d flex h-8 items-center gap-1.5 border-t px-3 text-[11.5px]">
          <Icon name="folder_open" size={13} weight={300} />
          <span data-part="captureTarget" className="truncate">
            {project && list ? `${project.name} / ${list.name}` : 'No list yet'}
          </span>
          {!notesOpen && (
            <span data-part="captureNotesHint" className="ml-auto shrink-0">
              Press Tab to add a note
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
