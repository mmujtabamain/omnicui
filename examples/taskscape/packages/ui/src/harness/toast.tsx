import { useEffect, useState } from "react";
import { Toast } from "../components/ui/toast";

function ToastHarness({ emit, open: initialOpen = false, ...props }: any) {
  const [open, setOpen] = useState<boolean>(initialOpen);
  useEffect(() => {
    setOpen(initialOpen);
  }, [initialOpen]);
  return (
    <Toast
      {...props}
      open={open}
      onOpenChange={(next: boolean) => {
        setOpen(next);
        emit("openChanged", { open: next });
      }}
      onDismissed={(reason: string) => emit("dismissed", { reason })}
      onActioned={(label: string) => emit("actioned", { label })}
    />
  );
}

export default { id: "ui.toast", Component: ToastHarness, events: ["openChanged", "dismissed", "actioned"] };
