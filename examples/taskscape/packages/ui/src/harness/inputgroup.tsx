import { Icon } from "../components/ui/icon";
import { InputGroup } from "../components/ui/input-group";

function InputGroupHarness(props: any) {
  const rest = { ...props };
  delete rest.emit;
  const search = rest.tone === "search";
  return (
    <InputGroup
      {...rest}
      inputLabel={search ? "Search tasks" : "Sheet name"}
      placeholder={search ? "Search" : "Name"}
      leading={search || props.shape === "bar" ? <Icon name="search" size={props.size === "sm" ? 14 : 16} /> : undefined}
    />
  );
}

export default { id: "ui.inputgroup", Component: InputGroupHarness, events: [] };
