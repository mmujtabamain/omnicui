import { SectionHeader } from "../components/ui/section-header";

function SectionHeaderHarness({ emit, ...props }: any) {
  return (
    <SectionHeader
      {...props}
      onRestore={() => emit("restoreRequested", { kind: props.restore })}
    />
  );
}

export default { id: "ui.sectionheader", Component: SectionHeaderHarness, events: ["restoreRequested"] };
