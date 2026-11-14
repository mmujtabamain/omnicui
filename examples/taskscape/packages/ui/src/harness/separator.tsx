import { Separator } from "../components/ui/separator";

/** Demo content around the rule, so each orientation and inset reads as it does in the app. */
function SeparatorHarness({ orientation, inset, depth, decorative, length, emphasis, className }: any) {
  const rule = (
    <Separator
      orientation={orientation}
      inset={inset}
      depth={depth}
      decorative={decorative}
      length={length}
      emphasis={emphasis}
      className={className}
    />
  );

  if (orientation === "vertical")
    return (
      <div className="flex h-[3rem] w-[18rem] items-center gap-[0.571rem] rounded-(--radius-field) bg-surface-3 px-[0.714rem] text-[0.857rem] text-content-2">
        <span>Bold</span>
        {rule}
        <span>Italic</span>
      </div>
    );

  if (inset === "row")
    return (
      <div className="relative flex h-[2.5rem] w-[18rem] items-center bg-surface-2 px-[0.857rem] text-[0.929rem] text-content-1">
        <span>Draft the brief</span>
        {rule}
      </div>
    );

  if (inset === "label")
    return (
      <div className="flex w-[18rem] items-center gap-[0.571rem]">
        <span className="text-[0.786rem] font-medium uppercase text-content-3">Attachments</span>
        {rule}
      </div>
    );

  if (inset === "menu")
    return (
      <div className="w-[18rem] rounded-(--radius-panel) bg-surface-3 py-[0.429rem] text-[0.929rem] text-content-1 shadow-menu">
        <div className="px-[0.857rem] py-[0.357rem]">Rename</div>
        {rule}
        <div className="px-[0.857rem] py-[0.357rem]">Delete</div>
      </div>
    );

  return (
    <div className="w-[18rem] rounded-(--radius-panel) border border-edge-1 bg-surface-2 text-[0.929rem] text-content-1">
      <div className="px-[0.857rem] py-[0.571rem]">Pane header</div>
      {rule}
      <div className="px-[0.857rem] py-[0.571rem] text-content-2">Pane body</div>
    </div>
  );
}

export default { id: "ui.separator", Component: SeparatorHarness, events: [] as string[] };
