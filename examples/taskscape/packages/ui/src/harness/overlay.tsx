import { useEffect, useState } from "react";
import { Button } from "../components/ui/button";
import { Overlay } from "../components/ui/overlay";

function OverlayHarness({ emit, open: initialOpen = false, ...props }: any) {
  const [open, setOpen] = useState<boolean>(initialOpen);
  useEffect(() => setOpen(initialOpen), [initialOpen]);

  return (
    <div className="flex items-start gap-2 p-3">
      <Button
        data-slot="opener"
        variant="outline"
        label="Open"
        onClick={() => {
          setOpen(true);
          emit("openChanged", { open: true });
        }}
      />
      <Button data-slot="outside" variant="outline" label="Outside" />
      <Overlay
        {...props}
        open={open}
        title="Quick open"
        onOpenChange={(next: boolean) => {
          setOpen(next);
          emit("openChanged", { open: next });
        }}
        onDismiss={() => {
          setOpen(false);
          emit("dismissed", {});
        }}
      >
        <div className="flex flex-col gap-[0.571rem]">
          <input
            data-slot="field"
            aria-label="Find a task"
            placeholder="Find a task"
            className="h-[2.071rem] w-full rounded-(--radius-field) border border-edge-2 bg-surface-3 px-[0.714rem] text-[0.929rem] text-content-1 placeholder:text-content-3 outline-none focus-visible:ring-2 focus-visible:ring-focus-1"
          />
          <div className="flex justify-end">
            <Button data-slot="action" variant="primary" label="Apply" />
          </div>
        </div>
      </Overlay>
    </div>
  );
}

export default { id: "ui.overlay", Component: OverlayHarness, events: ["openChanged", "dismissed"] };
