import { useState } from "react";
import * as AlertDialogPrimitive from "@radix-ui/react-alert-dialog";
import { DialogButton } from "../components/ui/dialog-button";

// accept and cancel are AlertDialog parts: cancel needs an AlertDialogContent above it, and accept needs the
// dialog root. The content is portalled into this harness so the buttons stay inside the mount, the way a
// sheet footer would.
function DialogButtonHarness({ emit, role, insteadId, ...props }: any) {
  const [host, setHost] = useState<HTMLElement | null>(null);
  return (
    <div ref={setHost}>
      {host && (
        <AlertDialogPrimitive.Root defaultOpen>
          <AlertDialogPrimitive.Portal container={host}>
            <AlertDialogPrimitive.Content className="flex justify-end" onOpenAutoFocus={(event) => event.preventDefault()}>
              <DialogButton
                role={role}
                insteadId={insteadId}
                {...props}
                onActivate={() => emit("activate", { role: "accept" })}
                onDismiss={() => emit("dismiss", { role: "cancel" })}
                onResolve={(id: string) => emit("resolve", { insteadId: id })}
              />
            </AlertDialogPrimitive.Content>
          </AlertDialogPrimitive.Portal>
        </AlertDialogPrimitive.Root>
      )}
    </div>
  );
}

export default {
  id: "ui.dialogbutton",
  Component: DialogButtonHarness,
  events: ["activate", "dismiss", "resolve"],
};
