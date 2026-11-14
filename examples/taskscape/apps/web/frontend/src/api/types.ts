// The shapes of the API (apps/web/backend/API.md).
export type Id = number;
export interface Me { id: Id; email: string; name: string }
export interface Settings {
  theme: "system" | "light" | "dark";
  reduced_motion: "system" | "on" | "off";
  show_completed: boolean;
  last_project: Id | null;
  last_list: Id | null;
}
export interface Binding { command: string; accel: string }
export interface Project { id: Id; name: string; sort_order: number; created_at: string; updated_at: string }
export interface TaskList {
  id: Id; project: Id; name: string; sort_order: number; created_at: string; updated_at: string;
  task_count: number; done_count: number;
}
export interface Note { id: Id; task: Id; content: string; sort_order: number; created_at: string; updated_at: string }
export interface Attachment { id: Id; task: Id; kind: "file" | "link"; name: string; url: string; size: number; mime: string; created_at: string }
export interface Task {
  id: Id; list: Id; parent: Id | null; title: string; notes: string; done: boolean; done_at: string | null;
  sort_order: number; created_at: string; updated_at: string; attachments: Attachment[]; note_items: Note[];
}
export interface TrashItem { id: Id; title: string; list: Id; list_name: string; deleted_at: string }
export interface ExportedTask { title: string; notes?: string; done?: boolean; children?: ExportedTask[] }
