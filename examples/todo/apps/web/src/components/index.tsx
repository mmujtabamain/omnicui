// The app's screens, as a library: what the app itself uses (main.tsx), and what the omnicui web
// target imports. Each is self-contained: it makes its store from its props (the seed and the
// settings) and reports what the user does through its on… callbacks, typed by the contracts'
// generated stubs (../omnicui).
import "./todo.css";
import { useState, type ReactNode } from "react";
import { seedTasks, TodoStore, useStore, type TodoEvent } from "../model.ts";
import { AppDefaults, type AppProps } from "../omnicui/app.ts";
import { DetailDefaults, type DetailProps } from "../omnicui/detail.ts";
import { ListDefaults, type ListProps } from "../omnicui/list.ts";
import { SettingsDefaults, type SettingsProps } from "../omnicui/settings.ts";
import { TaskDetailPane } from "./TaskDetail.tsx";
import { TaskListPane } from "./TaskList.tsx";
import { SettingsPage } from "./Settings.tsx";
import { Themed } from "./Themed.tsx";
import { TodoApp as App } from "./TodoApp.tsx";

export { TodoStore, seedTasks } from "../model.ts";

/** A store made once from the props, which reports each event to the matching on… prop. */
function useOwnStore(props: Record<string, unknown>, options: ConstructorParameters<typeof TodoStore>[1], seed: string) {
  const [store] = useState(() => new TodoStore(seedTasks(seed as never), options, (e: TodoEvent) => {
    const handler = props["on" + e.name[0].toUpperCase() + e.name.slice(1)];
    if (typeof handler === "function") handler(e.fields);
  }));
  return store;
}

function Screen({ store, children }: { store: TodoStore; children: ReactNode }) {
  const state = useStore(store);
  return <Themed theme={state.theme} className="screen">{children}</Themed>;
}

export function TodoList(props: ListProps) {
  const p = { ...ListDefaults, ...props };
  const store = useOwnStore(props as Record<string, unknown>, { showCompleted: p.showCompleted }, p.seed);
  return <Screen store={store}><TaskListPane store={store} /></Screen>;
}

export function TodoDetail(props: DetailProps) {
  const p = { ...DetailDefaults, ...props };
  const store = useOwnStore(props as Record<string, unknown>, {}, p.seed);
  return <Screen store={store}><TaskDetailPane store={store} id={p.taskId} /></Screen>;
}

export function TodoSettings(props: SettingsProps) {
  const p = { ...SettingsDefaults, ...props };
  const store = useOwnStore(props as Record<string, unknown>, { theme: p.theme, showCompleted: p.showCompleted }, "empty");
  return <Screen store={store}><SettingsPage store={store} /></Screen>;
}

export function TodoApp(props: AppProps) {
  const p = { ...AppDefaults, ...props };
  const store = useOwnStore(props as Record<string, unknown>, {}, p.seed);
  return <App store={store} />;
}
