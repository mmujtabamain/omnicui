// The task inspector: one task's title, done toggle, breadcrumb and times, notes, subtasks and
// attachments. It shows the task in the store's preview slot, in the list open in the active pane.
import * as React from "react";
import { Button } from "@taskscape/ui/button";
import { Checkbox } from "@taskscape/ui/checkbox";
import { Icon } from "@taskscape/ui/icon";
import { Input } from "@taskscape/ui/input";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { useLists, useProjects, useTasks, useWrite } from "../../api/queries";
import type { Attachment, Id, Task } from "../../api/types";
import { useUi } from "../../state/ui";
import { AttachmentLightbox } from "../lightbox/AttachmentLightbox";
import { AttachmentsSection } from "./AttachmentsSection";
import { type Emit, noEmit } from "./events";
import { NotesSection } from "./NotesSection";
import { SubtasksSection } from "./SubtasksSection";
import { absoluteDateTime, relativeTime } from "./time";

const P = "app.taskinspector.";

export function TaskInspector({ emit = noEmit }: { emit?: Emit }) {
  const previewId = useUi((s) => s.previewId);
  const listId = useUi((s) => s.panes[s.activePane] ?? null);
  const projectId = useUi((s) => s.projectId);
  const tasks = useTasks({ list: listId });
  const task = tasks.data?.find((t) => t.id === previewId);
  if (!task || listId === null) return null;
  return (
    <Inspector
      key={task.id}
      task={task}
      tasks={tasks.data ?? []}
      listId={listId}
      projectId={projectId}
      emit={emit}
    />
  );
}

function MetaRow({ icon, children }: { icon: string; children: React.ReactNode }) {
  return (
    <div className="flex min-w-0 items-center gap-[0.357rem] leading-[1.232rem]">
      <Icon name={icon} size={13} tone="inherit" className="shrink-0" />
      {children}
    </div>
  );
}

function Inspector({
  task,
  tasks,
  listId,
  projectId,
  emit,
}: {
  task: Task;
  tasks: Task[];
  listId: Id;
  projectId: Id | null;
  emit: Emit;
}) {
  const projects = useProjects();
  const lists = useLists(projectId);
  const updateTask = useWrite(ep.tasks.update);
  const projectName = projects.data?.find((p) => p.id === projectId)?.name ?? "-";
  const listName = lists.data?.find((l) => l.id === listId)?.name;
  const children = tasks.filter((t) => t.parent === task.id);

  const [title, setTitle] = React.useState(task.title);
  const [renameRequest, setRenameRequest] = React.useState(0);
  const [done, setDone] = React.useState(task.done);
  const [copied, setCopied] = React.useState(false);
  const [lightbox, setLightbox] = React.useState<Id | null>(null);

  React.useEffect(() => setTitle(task.title), [task.title]);
  React.useEffect(() => setDone(task.done), [task.done]);
  // The lightbox is a portal of its own; its content (data-slot lightbox) carries the lightbox part while open.
  React.useEffect(() => {
    if (lightbox === null) return;
    const part = `${P}lightbox`;
    const tag = () => document.querySelector('[data-slot="lightbox"]')?.setAttribute("data-part", part);
    tag();
    const observer = new MutationObserver(tag);
    observer.observe(document.body, { childList: true, subtree: true });
    return () => {
      observer.disconnect();
      document.querySelector(`[data-part="${part}"]`)?.removeAttribute("data-part");
    };
  }, [lightbox]);
  React.useEffect(() => {
    if (!copied) return;
    const timer = window.setTimeout(() => setCopied(false), 1200);
    return () => window.clearTimeout(timer);
  }, [copied]);

  const commitTitle = (text: string) => {
    if (text === "" || text === task.title) {
      setTitle(task.title);
      return;
    }
    setTitle(text);
    updateTask.mutate([task.id, { title: text }]);
    emit("titleChanged", { title: text });
  };

  const toggleDone = (next: boolean) => {
    setDone(next);
    updateTask.mutate([task.id, { done: next }]);
    emit("doneChanged", { title: task.title, done: next });
  };

  const copyTask = async () => {
    try {
      const { text } = await ep.tasks.copy(task.id);
      await navigator.clipboard.writeText(text || task.title);
    } catch {
      // Clipboard errors are swallowed, as the reference does; the button still confirms.
    }
    setCopied(true);
  };

  const openAttachment = (attachment: Attachment) => {
    emit("attachmentAction", { action: "open", name: attachment.name });
    if (attachment.kind === "link") window.open(attachment.url, "_blank", "noopener");
    else setLightbox(attachment.id);
  };

  const mentions = task.attachments.map((a) => ({ id: String(a.id), name: a.name }));
  const onOpenMention = (id: string) => {
    const attachment = task.attachments.find((a) => String(a.id) === id);
    if (attachment) openAttachment(attachment);
  };
  const onRequestLink = () => emit("linkRequested", { where: "note" });

  return (
    <section className="flex h-full min-h-0 flex-col bg-surface-1 text-content-1">
      <header className="relative flex items-start gap-[0.214rem] pl-[0.929rem] pr-[0.643rem] pt-[0.857rem]">
        <span
          data-part={`${P}accentTick`}
          aria-hidden="true"
          className="absolute left-0 top-[1.143rem] h-4 w-0.5 bg-accent-500"
        />
        <Checkbox
          data-part={`${P}doneToggle`}
          size="sm"
          intent="done"
          ariaLabel={done ? "Mark not done" : "Mark done"}
          checked={done}
          onCheckedChange={toggleDone}
          className="mt-[0.214rem] shrink-0"
        />
        <div className="flex min-w-0 flex-1 flex-col">
          <Input
            data-part={`${P}title`}
            ariaLabel="Title"
            multiline
            size="display"
            placeholder="Untitled"
            spellCheck={false}
            value={title}
            editRequest={renameRequest}
            onValueChange={setTitle}
            onCommit={commitTitle}
            onCancel={() => setTitle(task.title)}
          />
          <div className="mt-[0.286rem] flex flex-col gap-[0.071rem] pl-[0.286rem] text-content-3">
            <MetaRow icon="folder_open">
              <Text
                as="div"
                data-part={`${P}breadcrumb`}
                size="xs"
                tone="secondary"
                overflow="truncate"
                className="min-w-0"
              >
                {projectName}
                {listName !== undefined && <span className="text-content-3"> / {listName}</span>}
              </Text>
            </MetaRow>
            <MetaRow icon="schedule">
              <Text
                as="div"
                data-part={`${P}created`}
                title={absoluteDateTime(task.created_at)}
                size="xs"
                tone="muted"
                overflow="truncate"
              >
                {`Created ${relativeTime(task.created_at)}`}
              </Text>
            </MetaRow>
            <MetaRow icon="update">
              <Text
                as="div"
                data-part={`${P}updated`}
                title={absoluteDateTime(task.updated_at)}
                size="xs"
                tone="muted"
                overflow="truncate"
              >
                {`Updated ${relativeTime(task.updated_at)}`}
              </Text>
            </MetaRow>
          </div>
        </div>
        <div className="flex flex-col gap-[0.143rem] pt-[0.357rem]">
          <Button
            data-part={`${P}closePanel`}
            ariaLabel="Close panel"
            description="Close panel"
            icon="last_page"
            iconSize={16}
            variant="ghost"
            size="sm"
            shape="icon"
            tone="muted"
            onClick={() => emit("previewOpenChanged", { open: false })}
          />
          <Button
            data-part={`${P}rename`}
            ariaLabel="Rename"
            description="Rename"
            icon="edit"
            iconSize={15}
            variant="ghost"
            size="sm"
            shape="icon"
            tone="muted"
            onClick={() => setRenameRequest((n) => n + 1)}
          />
          <Button
            data-part={`${P}copyTask`}
            ariaLabel={copied ? "Copied" : "Copy task"}
            description={copied ? "Copied" : "Copy task"}
            icon={copied ? "check" : "content_copy"}
            iconSize={15}
            variant="ghost"
            size="sm"
            shape="icon"
            tone="muted"
            onClick={() => void copyTask()}
          />
          <Button
            data-part={`${P}deleteTask`}
            ariaLabel="Delete task"
            description="Delete task"
            icon="delete"
            iconSize={15}
            variant="ghost"
            size="sm"
            shape="icon"
            tone="muted"
            onClick={() => emit("deleteRequested", { target: "task", title: task.title })}
          />
        </div>
      </header>

      <div className="flex min-h-0 flex-1 flex-col gap-[1.143rem] overflow-y-auto px-[0.929rem] pb-[0.929rem] pt-[0.929rem]">
        <NotesSection
          task={task}
          mentions={mentions}
          emit={emit}
          onOpenMention={onOpenMention}
          onRequestLink={onRequestLink}
        />
        {children.length > 0 && <SubtasksSection subtasks={children} tasks={tasks} emit={emit} />}
        <AttachmentsSection attachments={task.attachments} emit={emit} onOpen={openAttachment} />
      </div>

      {lightbox !== null && (
        <AttachmentLightbox
          listId={listId}
          taskId={task.id}
          attachmentId={lightbox}
          events={{ closed: () => setLightbox(null) }}
        />
      )}
    </section>
  );
}
