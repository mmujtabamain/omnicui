// Generated from examples/todo/desktop/omnicui/contracts/list.contract.yaml by stubs.ts. Do not edit: change the contract instead.

export interface ListAdded { id: string; title: string; }
export interface ListToggled { id: string; done: boolean; }
export interface ListOpened { id: string; }
export interface ListFilterChanged { filter: "all" | "active" | "done"; }
export interface ListProps {
  seed?: "empty" | "three" | "many";
  showCompleted?: boolean;
  onAdded?: (event: ListAdded) => void;
  onToggled?: (event: ListToggled) => void;
  onOpened?: (event: ListOpened) => void;
  onFilterChanged?: (event: ListFilterChanged) => void;
}
export const ListDefaults = { seed: "three", showCompleted: true } as const;
export const Parts = { tasksHeading: "todo.list.tasksHeading", filterAll: "todo.list.filterAll", filterActive: "todo.list.filterActive", filterDone: "todo.list.filterDone", left: "todo.list.left", add: "todo.list.add", row1: "todo.list.row1", row2: "todo.list.row2", row3: "todo.list.row3", row1Check: "todo.list.row1Check", row2Check: "todo.list.row2Check", row3Check: "todo.list.row3Check", empty: "todo.list.empty", newTaskDialog: "todo.list.newTaskDialog", newTitle: "todo.list.newTitle", confirmAdd: "todo.list.confirmAdd", cancelAdd: "todo.list.cancelAdd" } as const;
export const Events = ["added", "toggled", "opened", "filterChanged"] as const;
