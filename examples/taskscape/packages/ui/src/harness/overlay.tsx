import { useState } from "react";
import { Button } from "../components/ui/button";
import { Overlay, OverlayTitle } from "../components/ui/overlay";

function OverlayHarness({ emit, open: initialOpen, ...props }: any) {
  const [open, setOpen] = useState(Boolean(initialOpen));
  // A fill layer is the whole window: its fixture column is centred and kept narrow.
  const column = props.placement === "fill" ? "mx-auto flex h-full w-[min(18rem,100%)] flex-col justify-center gap-2 p-3" : "flex flex-col gap-2 p-3";
  return (
    <>
      <div className="fixed top-3 left-3 flex flex-col items-start gap-2">
        <Button data-slot="opener" variant="outline" label="Open layer" onClick={() => setOpen(true)} />
      </div>
      {/* Above the scrim, so a press on it is a press on the page outside the layer. */}
      <div className="fixed bottom-3 left-3 z-(--z-index-modal)">
        <Button data-slot="outside" variant="outline" label="Outside" style={{ pointerEvents: "auto" }} />
      </div>
      <Overlay
        {...props}
        open={open}
        onOpenChange={(next: boolean) => {
          emit("openChanged", { open: next });
          setOpen(next);
        }}
        onDismiss={() => emit("dismissed", {})}
      >
        <div className={column}>
          <OverlayTitle>Overlay</OverlayTitle>
          <input
            data-slot="field"
            aria-label="Name"
            placeholder="Name"
            className="h-[1.75rem] w-full min-w-0 rounded-(--radius-field) border border-edge-2 bg-surface-3 px-2 text-[0.929rem] text-content-1 outline-none focus-visible:ring-2 focus-visible:ring-focus-1"
          />
          <div className="flex justify-end">
            <Button data-slot="action" variant="primary" label="Done" />
          </div>
        </div>
      </Overlay>
    </>
  );
}

export default { id: "ui.overlay", Component: OverlayHarness, events: ["openChanged", "dismissed"] };
