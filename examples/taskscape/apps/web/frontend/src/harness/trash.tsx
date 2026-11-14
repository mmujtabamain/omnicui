// The harness entry for the Trash pane (contract app.trash): the pane alone, in the page the demo
// data set and account are already loaded into. staleTitle changes the server behind the pane once
// its list has rendered, without the pane knowing, so the next restore or purge of that row fails.
import { useEffect, useRef } from "react";
import { useMutation } from "@tanstack/react-query";
import * as ep from "../api/endpoints";
import { useTrash } from "../api/queries";
import { TrashPane } from "../screens/trash/TrashPane";
import { AppHarness } from "./lib/AppHarness";

type Emit = (name: string, fields: Record<string, unknown>) => void;

/** Purges the trashed root with this title, once the list has loaded. No UI, no cache update. */
function StaleChange({ title }: { title: string }) {
  const trash = useTrash();
  const { mutate } = useMutation({ mutationFn: (ids: number[]) => ep.trash.purge(ids) });
  const done = useRef(false);

  useEffect(() => {
    if (done.current || !trash.data) return;
    const item = trash.data.find((row) => row.title === title);
    if (!item) return;
    done.current = true;
    mutate([item.id]);
  }, [trash.data, title, mutate]);

  return null;
}

function TrashHarness({ staleTitle = "", emit }: { dataset?: string; staleTitle?: string; emit?: Emit }) {
  return (
    <AppHarness>
      <div className="flex h-dvh w-full">
        <TrashPane
          onClose={() => emit?.("closed", { pane: "trash" })}
          onRestored={(count) => emit?.("restored", { count })}
          onPurged={(count) => emit?.("purged", { count })}
        />
        {staleTitle !== "" && <StaleChange title={staleTitle} />}
      </div>
    </AppHarness>
  );
}

export default { id: "app.trash", Component: TrashHarness, events: ["restored", "purged", "closed"] };
