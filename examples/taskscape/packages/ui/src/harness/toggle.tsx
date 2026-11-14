import { Toggle } from "../components/ui/toggle";

function ToggleHarness({ emit, ...props }: any) {
  return <Toggle {...props} onPressedChange={(pressed: boolean) => emit("pressedChange", { pressed })} />;
}

export default { id: "ui.toggle", Component: ToggleHarness, events: ["pressedChange"] };
