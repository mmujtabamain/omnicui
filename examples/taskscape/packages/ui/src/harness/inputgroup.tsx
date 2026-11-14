import { Icon } from "../components/ui/icon";
import { InputGroup, InputGroupInput } from "../components/ui/input-group";

function InputGroupHarness(props: any) {
  const { shape = "well", tone = "field", ...rest } = props;
  delete rest.emit; // the frame emits no events
  const search = tone === "search";
  const leading = shape === "bar" || search ? <Icon name="search" size="md" /> : undefined;
  return (
    <InputGroup shape={shape} tone={tone} leading={leading} {...rest}>
      <InputGroupInput
        aria-label={search ? "Search tasks" : "Name"}
        placeholder={search ? "Search" : "Name"}
      />
    </InputGroup>
  );
}

export default { id: "ui.inputgroup", Component: InputGroupHarness, events: [] };
