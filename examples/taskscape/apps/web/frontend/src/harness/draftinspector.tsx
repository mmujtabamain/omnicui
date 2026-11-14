import * as React from "react";
import { AppHarness } from "./lib/AppHarness";
import { DraftInspector } from "../screens/draft-inspector/DraftInspector";
import { useLists, useProjects } from "../api/queries";

interface HarnessProps {
  dataset?: string;
  list?: string;
  emit?: (name: string, payload: unknown) => void;
}

function Mounted({ list = "Errands", emit }: HarnessProps) {
  const projects = useProjects();
  const home = projects.data?.find((p) => p.name === "Home");
  const lists = useLists(home?.id ?? null);
  const [open, setOpen] = React.useState(true);

  if (!home) return null;
  if (list !== "" && !lists.data) return null;
  const target = list === "" ? null : (lists.data?.find((l) => l.name === list) ?? null);
  if (list !== "" && !target) return null;
  if (!open) return null;

  return (
    <DraftInspector
      listId={target?.id ?? null}
      projectName={home.name}
      listName={target?.name ?? null}
      onAction={(action, title) => {
        emit?.("draftAction", { action, title });
        setOpen(false);
      }}
      onImageAdded={() => setOpen(false)}
    />
  );
}

function DraftInspectorHarness(props: HarnessProps) {
  return (
    <AppHarness>
      <div className="flex h-full min-h-0 w-full flex-col">
        <Mounted {...props} />
      </div>
    </AppHarness>
  );
}

export default { id: "app.draftinspector", Component: DraftInspectorHarness, events: ["draftAction"] };
