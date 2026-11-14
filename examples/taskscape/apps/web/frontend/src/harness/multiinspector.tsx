// The harness entry for app.multiinspector: selects the named tasks in the named list (by title, in
// list order), then mounts MultiInspector. Renders nothing until the selection is set. The whole
// set-up is one query, so the page counts as busy until the store and the caches are ready.
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { TooltipProvider } from "@taskscape/ui/tooltip";
import { AppHarness } from "./lib/AppHarness";
import * as ep from "../api/endpoints";
import { keys } from "../api/queries";
import { useUi } from "../state/ui";
import { MultiInspector } from "../screens/multi-inspector/MultiInspector";

/** Selected task titles per starting state, in list order (contract: selection four | two). */
const SELECTIONS: Record<string, string[]> = {
  four: ["Pick up prescription", "Post the parcel to Mum", "Book bike service", "Cut a spare key"],
  two: ["Book bike service", "Cut a spare key"],
};

function Setup({ list, selection }: { list: string; selection: string }) {
  const qc = useQueryClient();
  const setup = useQuery({
    queryKey: ["harness", "app.multiinspector", list, selection],
    retry: false,
    queryFn: async () => {
      const projects = await ep.projects.list();
      const home = projects.find((p) => p.name === "Home");
      if (!home) throw new Error("The demo data has no Home project");
      const lists = await ep.lists.list(home.id);
      const target = lists.find((l) => l.name === list);
      if (!target) throw new Error(`The demo data has no list named ${list}`);
      const tasks = await ep.tasks.list({ list: target.id });

      // Fill the caches the screen reads, so it renders from them without a further request.
      qc.setQueryData(keys.projects, projects);
      qc.setQueryData(keys.lists(home.id), lists);
      qc.setQueryData(keys.tasks({ list: target.id }), tasks);

      const titles = SELECTIONS[selection] ?? SELECTIONS.four;
      const picked = tasks
        .filter((t) => t.parent == null && titles.includes(t.title))
        .sort((a, b) => a.sort_order - b.sort_order);
      useUi.setState({
        projectId: home.id,
        panes: [target.id],
        activePane: 0,
        selection: picked.map((t) => t.id),
        // The last selected task is the active one: the reference screenshots highlight it.
        previewId: picked[picked.length - 1]?.id ?? null,
      });
      return true;
    },
  });

  if (!setup.data) return null;
  return <MultiInspector />;
}

function MultiInspectorHarness(props: { dataset?: string; list?: string; selection?: string }) {
  return (
    <AppHarness>
      <TooltipProvider>
        <div className="flex h-full w-full min-w-0">
          <Setup list={props.list ?? "Errands"} selection={props.selection ?? "four"} />
        </div>
      </TooltipProvider>
    </AppHarness>
  );
}

export default { id: "app.multiinspector", Component: MultiInspectorHarness, events: [] };
