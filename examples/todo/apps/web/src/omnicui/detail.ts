// Generated from examples/todo/omnicui/contracts/detail.contract.yaml by stubs.ts. Do not edit: change the contract instead.

export interface DetailRenamed { id: string; title: string; }
export interface DetailNotesChanged { id: string; notes: string; }
export interface DetailToggled { id: string; done: boolean; }
export interface DetailDeleted { id: string; }
export interface DetailClosed { id: string; }
export interface DetailProps {
  seed?: "empty" | "three" | "many";
  taskId?: string;
  onRenamed?: (event: DetailRenamed) => void;
  onNotesChanged?: (event: DetailNotesChanged) => void;
  onToggled?: (event: DetailToggled) => void;
  onDeleted?: (event: DetailDeleted) => void;
  onClosed?: (event: DetailClosed) => void;
}
export const DetailDefaults = { seed: "three", taskId: "t1" } as const;
export const Parts = { title: "todo.detail.title", notes: "todo.detail.notes", done: "todo.detail.done", delete: "todo.detail.delete", detailBack: "todo.detail.detailBack" } as const;
export const Events = ["renamed", "notesChanged", "toggled", "deleted", "closed"] as const;
