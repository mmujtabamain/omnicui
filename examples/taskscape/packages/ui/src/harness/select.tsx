import { Select } from "../components/ui/select";

// The value stays as the test sets it: the widget is controlled, so a pick only emits.
function SelectHarness({ emit, ...props }: any) {
  return <Select {...props} onValueChange={(value: string) => emit("valueChanged", { value })} />;
}

export default { id: "ui.select", Component: SelectHarness, events: ["valueChanged"] };
