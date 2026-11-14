import { useState } from "react";
import { Dialog } from "../components/ui/dialog";

function DialogHarness({ emit, open: openProp = false, ...props }: any) {
  const [open, setOpen] = useState<boolean>(Boolean(openProp));
  return (
    <Dialog
      {...props}
      open={open}
      onOpenChange={(next: boolean) => {
        setOpen(next);
        emit("openChanged", { open: next });
      }}
      onAccept={() => emit("accepted", {})}
    />
  );
}

export default { id: "ui.dialog", Component: DialogHarness, events: ["openChanged", "accepted"] };
