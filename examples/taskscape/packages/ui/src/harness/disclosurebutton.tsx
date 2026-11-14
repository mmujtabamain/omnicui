import { useState } from "react";
import { DisclosureButton } from "../components/ui/disclosure-button";

/** A task row: the control sits left of the title and the subtask counter. */
function DisclosureButtonHarness({ emit, expanded, hidden, ...props }: any) {
  const [open, setOpen] = useState<boolean>(expanded ?? false);
  const change = (next: boolean) => {
    setOpen(next);
    emit("expandedChanged", { expanded: next });
  };
  return (
    <div
      data-slot="row"
      className="group/row flex w-80 items-center gap-2 rounded-(--radius-field) px-2 py-1.5 text-[0.929rem] text-content-1 hover:bg-wash-1"
      onClick={() => emit("rowOpened", {})}
    >
      <DisclosureButton expanded={open} hidden={hidden} onExpandedChange={change} {...props} />
      <span className="min-w-0 flex-1 truncate">Write the quarterly report</span>
      <span className="text-[0.857rem] text-content-3">2/5</span>
    </div>
  );
}

export default { id: "ui.disclosurebutton", Component: DisclosureButtonHarness, events: ["expandedChanged", "rowOpened"] };
