import { useState } from "react";
import { Switch } from "../components/ui/switch";

function SwitchHarness({ emit, checked: initial, ...props }: any) {
  const [checked, setChecked] = useState<boolean>(Boolean(initial));
  return (
    <Switch
      {...props}
      checked={checked}
      onCheckedChange={(next: boolean) => {
        setChecked(next);
        emit("changed", { checked: next });
      }}
    />
  );
}

export default { id: "ui.switch", Component: SwitchHarness, events: ["changed"] };
