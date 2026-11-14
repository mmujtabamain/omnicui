import { useState } from "react";
import { Lightbox, LightboxMedia, type LightboxCloseReason } from "../components/ui/lightbox";

function LightboxHarness({ emit, open: openProp = false, presentation, ...props }: any) {
  const [open, setOpen] = useState<boolean>(openProp);
  return (
    <Lightbox
      {...props}
      presentation={presentation}
      open={open}
      onClose={(reason: LightboxCloseReason) => {
        setOpen(false);
        emit("close", { reason });
      }}
    >
      <LightboxMedia>
        <div className="flex w-full max-w-[28rem] flex-col gap-2 rounded-(--radius-panel) bg-surface-2 p-3 shadow-menu">
          <div className="flex aspect-[4/3] items-center justify-center rounded-(--radius-control) bg-wash-2 text-[0.929rem] text-content-3">
            Preview
          </div>
          <div className="flex min-w-0 flex-col gap-0.5 px-1">
            <span className="truncate text-[1rem] font-semibold text-content-1">Q3 plan.pdf</span>
            <span className="text-[0.857rem] text-content-2">PDF · 2.4 MB · Added today</span>
          </div>
        </div>
      </LightboxMedia>
    </Lightbox>
  );
}

export default { id: "ui.lightbox", Component: LightboxHarness, events: ["close"] };
