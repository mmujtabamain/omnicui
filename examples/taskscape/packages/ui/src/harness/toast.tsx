import { useEffect, useRef, useState } from "react";
import { Toast } from "../components/ui/toast";

function ToastHarness({ emit, open: initialOpen = false, awaitTimeout = false, ...props }: any) {
  const [open, setOpen] = useState<boolean>(initialOpen);
  useEffect(() => {
    setOpen(initialOpen);
  }, [initialOpen]);
  // awaitTimeout: the test watches the toast time out, so until it closes the harness is busy and
  // omnicui waits for it.
  const openNow = useRef(open);
  openNow.current = open;
  useEffect(() => {
    const w = window as any;
    if (awaitTimeout) w.omnicuiBusy = () => openNow.current;
    return () => { delete w.omnicuiBusy; };
  }, [awaitTimeout]);
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
