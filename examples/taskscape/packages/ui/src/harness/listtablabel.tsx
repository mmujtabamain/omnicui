import { ListTabLabel } from "../components/ui/list-tab-label";

function ListTabLabelHarness({ emit, ...props }: any) {
  return (
    <ListTabLabel
      {...props}
      onSelect={() => emit("select", {})}
      onClose={() => emit("close", {})}
    />
  );
}

export default { id: "ui.listtablabel", Component: ListTabLabelHarness, events: ["select", "close"] };
