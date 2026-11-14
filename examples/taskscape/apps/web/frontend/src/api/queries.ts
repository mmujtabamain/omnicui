// React Query keys and hooks over the endpoints. Writes invalidate what they change.
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as ep from "./endpoints";
import type { Id } from "./types";

export const keys = {
  me: ["me"] as const,
  settings: ["settings"] as const,
  shortcuts: ["shortcuts"] as const,
  projects: ["projects"] as const,
  lists: (project?: Id | null) => ["lists", project ?? "all"] as const,
  tasks: (scope: { list?: Id | null; project?: Id | null; q?: string }) => ["tasks", scope] as const,
  notes: (task: Id) => ["notes", task] as const,
  trash: ["trash"] as const,
};

export const useMe = () => useQuery({ queryKey: keys.me, queryFn: ep.auth.me, retry: false });
/** The account's settings; nothing is asked for while signed out (it would only answer 401). */
export const useSettings = () => {
  const me = useMe();
  return useQuery({ queryKey: keys.settings, queryFn: ep.settings.get, enabled: !!me.data });
};
export const useShortcuts = () => useQuery({ queryKey: keys.shortcuts, queryFn: ep.settings.shortcuts });
export const useProjects = () => useQuery({ queryKey: keys.projects, queryFn: ep.projects.list });
export const useLists = (project: Id | null | undefined) =>
  useQuery({ queryKey: keys.lists(project), queryFn: () => ep.lists.list(project ?? undefined), enabled: project != null });
export const useTasks = (scope: { list?: Id | null; project?: Id | null; q?: string }) =>
  useQuery({ queryKey: keys.tasks(scope), queryFn: () => ep.tasks.list({ list: scope.list ?? undefined, project: scope.project ?? undefined, q: scope.q }), enabled: scope.list != null || scope.project != null });
export const useTrash = () => useQuery({ queryKey: keys.trash, queryFn: ep.trash.list });

/** A write that refreshes everything task-shaped afterwards (lists' counts, tasks, trash). */
export function useWrite<A extends unknown[], R>(fn: (...a: A) => Promise<R>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (a: A) => fn(...a),
    onSettled: () => {
      for (const k of ["lists", "tasks", "notes", "trash", "projects"]) qc.invalidateQueries({ queryKey: [k] });
    },
  });
}
