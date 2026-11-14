// Mounts the attachment lightbox over the demo task's attachments, once the task has been found.
import { useQuery } from "@tanstack/react-query";
import * as ep from "../api/endpoints";
import type { AttachmentLightboxEvents } from "../screens/lightbox/AttachmentLightbox";
import { AttachmentLightbox } from "../screens/lightbox/AttachmentLightbox";
import { AppHarness } from "./lib/AppHarness";

const EVENT_NAMES = ["closed", "openedExternally", "renamed", "removed", "stepped"] as const;

type HostProps = {
  dataset?: string;
  /** Title of the task whose attachments the lightbox steps through. */
  task?: string;
  /** File name the lightbox opens on. */
  attachment?: string;
  emit?: (name: string, payload: object) => void;
  [key: string]: unknown;
};

/** Forwards each screen event to the host: a callback prop named for the event, its on-prop, or emit. */
function eventsFrom(props: HostProps): AttachmentLightboxEvents {
  const out: Record<string, (payload: object) => void> = {};
  for (const name of EVENT_NAMES) {
    const onName = `on${name[0].toUpperCase()}${name.slice(1)}`;
    const direct = props[name];
    const on = props[onName];
    const handler = typeof direct === "function" ? direct : typeof on === "function" ? on : undefined;
    out[name] = (payload) => {
      (handler as ((p: object) => void) | undefined)?.(payload);
      props.emit?.(name, payload);
    };
  }
  return out as AttachmentLightboxEvents;
}

/** Finds the list, the task and the attachment by name in the demo data set. */
async function resolve(taskTitle: string, attachmentName: string) {
  const projects = await ep.projects.list();
  const project = projects.find((p) => p.name === "Spring launch");
  if (!project) throw new Error("Project Spring launch is missing from the demo data");
  const lists = await ep.lists.list(project.id);
  const list = lists.find((l) => l.name === "Website");
  if (!list) throw new Error("List Website is missing from the demo data");
  const tasks = await ep.tasks.list({ list: list.id });
  const task = tasks.find((t) => t.title === taskTitle);
  if (!task) throw new Error(`Task "${taskTitle}" is missing from the demo data`);
  const attachment = task.attachments.find((a) => a.name === attachmentName);
  if (!attachment) throw new Error(`Attachment "${attachmentName}" is missing from the demo data`);
  return { listId: list.id, taskId: task.id, attachmentId: attachment.id };
}

function Resolved({ task, attachment, events }: { task: string; attachment: string; events?: AttachmentLightboxEvents }) {
  const found = useQuery({
    queryKey: ["harness", "attachmentlightbox", task, attachment],
    queryFn: () => resolve(task, attachment),
  });
  if (!found.data) return null;
  return (
    <AttachmentLightbox
      listId={found.data.listId}
      taskId={found.data.taskId}
      attachmentId={found.data.attachmentId}
      events={events}
    />
  );
}

function AttachmentLightboxHarness(props: HostProps) {
  const task = props.task ?? "Rewrite the hero section";
  const attachment = props.attachment ?? "hero-sketch.png";
  return (
    <AppHarness>
      <Resolved task={task} attachment={attachment} events={eventsFrom(props)} />
    </AppHarness>
  );
}

export default {
  id: "app.attachmentlightbox",
  Component: AttachmentLightboxHarness,
  events: ["closed", "openedExternally", "renamed", "removed", "stepped"],
};
