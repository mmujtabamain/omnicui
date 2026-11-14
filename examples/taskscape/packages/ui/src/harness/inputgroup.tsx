import { Icon } from "../components/ui/icon";
import { InputGroup } from "../components/ui/input-group";

function InputGroupHarness({ emit: _emit, ...props }: any) {
  const search = props.tone === "search";
  return (
    <InputGroup
      {...props}
      inputLabel={search ? "Search tasks" : "Sheet name"}
      placeholder={search ? "Search" : "Name"}
      leading={search || props.shape === "bar" ? <Icon name="search" size={props.size === "sm" ? 14 : 16} /> : undefined}
    />
  );
}

export default { id: "ui.inputgroup", Component: InputGroupHarness, events: [] };
