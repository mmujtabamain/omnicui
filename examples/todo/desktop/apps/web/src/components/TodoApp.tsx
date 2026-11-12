import { useEffect, useRef, useState } from "react";
import { useStore, type TodoStore } from "../model.ts";
import { Parts } from "../omnicui/app.ts";
import { GearIcon, TaskIcon } from "./icons.tsx";
import { SettingsPage } from "./Settings.tsx";
import { NoTask, TaskDetailPane } from "./TaskDetail.tsx";
import { TaskListPane } from "./TaskList.tsx";
import { Themed } from "./Themed.tsx";

/** Which screen is shown: the list (with no task open), a task, or settings. */
export type Route = { screen: "list" } | { screen: "detail"; id: string } | { screen: "settings" };

// Routes live in the query, so that the app works under any path: ?task=t1, ?settings.
const routeOf = (search: string): Route => {
  const q = new URLSearchParams(search);
  if (q.has("settings")) return { screen: "settings" };
  const id = q.get("task");
  return id ? { screen: "detail", id } : { screen: "list" };
};
const urlOf = (r: Route) => location.pathname + (r.screen === "settings" ? "?settings" : r.screen === "detail" ? `?task=${encodeURIComponent(r.id)}` : "");
const same = (a: Route, b: Route) => urlOf(a) === urlOf(b);

/**
 * The whole app: a header with Settings, the task list in a sidebar, the open task in a pane to
 * its right, and settings as a page of its own. Each change of screen is reported as `navigated`
 * (not the first one).
 */
export function TodoApp({ store }: { store: TodoStore }) {
  const state = useStore(store);
  const [route, setRoute] = useState<Route>(() => routeOf(location.search));
  const current = useRef(route);
  const show = (r: Route) => {
    if (same(r, current.current)) return;
    current.current = r;
    setRoute(r);
    store.report("navigated", { screen: r.screen, id: r.screen === "detail" ? r.id : "" });
  };
  useEffect(() => {
    const onPop = () => show(routeOf(location.search));
    addEventListener("popstate", onPop);
    return () => removeEventListener("popstate", onPop);
  }, []);
  // The list is one history entry, and whatever is open away from it (a task, settings) one more:
  // moving between a task and settings replaces it, so back always returns to the list.
  const go = (r: Route) => {
    if (same(r, current.current)) return;
    if (r.screen === "list") return toList();
    if (current.current.screen === "list") history.pushState({ todo: true }, "", urlOf(r));
    else history.replaceState({ todo: true }, "", urlOf(r));
    show(r);
  };
  const toList = () => {
    const list: Route = { screen: "list" };
    show(list);
    if (history.state?.todo) history.back();
    else history.replaceState(null, "", urlOf(list));
  };
  return (
    <Themed theme={state.theme} className="app">
      <header className="header">
        <span className="header__brand"><TaskIcon />Todo</span>
        <button type="button" className="text-button header__settings" aria-current={route.screen === "settings" ? "page" : undefined}
          data-part={Parts.openSettings} onClick={() => go({ screen: "settings" })}>
          <GearIcon />Settings
        </button>
      </header>
      {route.screen === "settings" ? (
        <SettingsPage store={store} onLeave={toList} />
      ) : (
        <div className="app__body">
          <TaskListPane store={store} current={route.screen === "detail" ? route.id : undefined} onOpen={(id) => go({ screen: "detail", id })} />
          {route.screen === "detail" ? <TaskDetailPane key={route.id} store={store} id={route.id} onLeave={toList} /> : <NoTask />}
        </div>
      )}
    </Themed>
  );
}
