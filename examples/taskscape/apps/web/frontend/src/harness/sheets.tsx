// The sheets contract: mounts SheetHost alone at 1180x760, signed in as the demo account. The
// harness finds the named list, project or task in the demo data, opens the sheet for `sheet`
// through openSheet, and reports its answer as the "answered" event.
import * as React from "react";
import { AppHarness } from "./lib/AppHarness";
import { useLists, useProjects, useTasks } from "../api/queries";
import { useUi } from "../state/ui";
import { SheetHost, openSheet, type SheetResult, type SheetSpec } from "../screens/sheets";

/** The answer the contract's "answered" event carries. */
interface Answered {
  action: string;
  text: string;
  id: string;
}

interface SheetsProps {
  dataset?: string;
  sheet?: string;
  list?: string;
  project?: string;
  task?: string;
  /** Receives the "answered" event: the sheet resolved. */
  onAnswered?: (payload: Answered) => void;
}

function SheetsHarness(props: SheetsProps) {
  return (
    <AppHarness>
      <SheetsStage {...props} />
    </AppHarness>
  );
}

function SheetsStage({
  sheet = "newList", list = "Reading", project = "Spring launch", task = "Book bike service", onAnswered,
}: SheetsProps) {
  const projects = useProjects();
  const home = projects.data?.find((p) => p.name === "Home");
  const homeLists = useLists(home?.id);
  const errands = homeLists.data?.find((l) => l.name === "Errands");
  const errandTasks = useTasks({ list: errands?.id });
  const started = React.useRef(false);

  // Renders nothing until the data the chosen sheet needs has loaded.
  const spec: SheetSpec | null = (() => {
    if (!projects.data || !homeLists.data || !home) return null;
    switch (sheet) {
      case "newList":
        return { kind: "newList", projectId: home.id };
      case "newProject":
        return { kind: "newProject" };
      case "renameList": {
        const found = homeLists.data.find((l) => l.name === list);
        return found ? { kind: "renameList", list: found } : null;
      }
      case "renameProject": {
        const found = projects.data.find((p) => p.name === project);
        return found ? { kind: "renameProject", project: found } : null;
      }
      case "deleteList": {
        const found = homeLists.data.find((l) => l.name === list);
        return found ? { kind: "deleteList", list: found } : null;
      }
      case "deleteProject": {
        const found = projects.data.find((p) => p.name === project);
        return found ? { kind: "deleteProject", project: found } : null;
      }
      case "deleteConfirm": {
        if (!errandTasks.data) return null;
        const found = errandTasks.data.find((t) => t.title === task);
        return found ? { kind: "deleteConfirm", task: found } : null;
      }
      default:
        return null;
    }
  })();

  React.useEffect(() => {
    if (!spec || started.current) return;
    started.current = true;
    if (spec.kind === "deleteConfirm" && home && errands) {
      // The task is selected in the list behind the sheet, as the reference shows it.
      const ui = useUi.getState();
      ui.setProject(home.id);
      ui.openList(errands.id);
      ui.select([spec.task.id]);
      ui.preview(spec.task.id);
    }
    openSheet(spec).then((result) => onAnswered?.(answered(result)));
  }, [spec, home, errands, onAnswered]);

  if (!spec) return null;
  return (
    <div className="fixed inset-0 bg-surface-1">
      <SheetHost />
    </div>
  );
}

/** The answer as the event carries it: text and id are "" when the result has none. */
function answered(result: SheetResult): Answered {
  return { action: result.action, text: result.text ?? "", id: result.id ?? "" };
}

export default { id: "app.sheets", Component: SheetsHarness, events: ["answered"] };
