// React Query keys and hooks over the endpoints. Writes invalidate what they change.
import { useMutation, useQuery, useQueryClient, type QueryClient } from "@tanstack/react-query";
import { useSheets } from "../screens/sheets/store";
import { applyOptimistic, restore } from "./optimistic";
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

const refresh = (qc: QueryClient) => {
  for (const k of ["lists", "tasks", "notes", "trash", "projects"]) qc.invalidateQueries({ queryKey: [k] });
};

/**
 * A write. The screen changes at once (api/optimistic.ts), the server is asked, and everything task-shaped is
 * refreshed afterwards (lists' counts, tasks, trash). A refused write puts the screen back as it was and says
 * so, with Try again.
 */
export function useWrite<A extends unknown[], R>(fn: (...a: A) => Promise<R>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (a: A) => fn(...a),
    onMutate: (a: A) => applyOptimistic(qc, fn, a),
    onError: (_error, a, snapshot) => {
      if (!snapshot) return;
      restore(qc, snapshot);
      useSheets.getState().setToast({
        message: "",
        failed: true,
        errorMessage: "Couldn't save that change",
        retry: async () => {
          const again = await applyOptimistic(qc, fn, a);
          try {
            await fn(...a);
          } catch (error) {
            if (again) restore(qc, again);
            throw error;
          } finally {
            refresh(qc);
          }
        },
      });
    },
    onSettled: () => refresh(qc),
  });
}
