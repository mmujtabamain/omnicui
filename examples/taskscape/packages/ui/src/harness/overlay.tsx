import { useState } from "react";
import { Button } from "../components/ui/button";
import { Overlay, OverlayTitle } from "../components/ui/overlay";

function OverlayHarness({ emit, open: initialOpen, ...props }: any) {
  const [open, setOpen] = useState(Boolean(initialOpen));
  return (
    <div className="flex flex-col items-start gap-2 p-3">
      <Button data-slot="opener" variant="outline" label="Open layer" onClick={() => setOpen(true)} />
      <Button data-slot="outside" variant="outline" label="Outside" className="pointer-events-auto" />
      <Overlay
        {...props}
        open={open}
        onOpenChange={(next: boolean) => {
          emit("openChanged", { open: next });
          setOpen(next);
        }}
        onDismiss={() => emit("dismissed", {})}
      >
        <div className="flex flex-col gap-2 p-3">
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
    </div>
  );
}

export default { id: "ui.overlay", Component: OverlayHarness, events: ["openChanged", "dismissed"] };
