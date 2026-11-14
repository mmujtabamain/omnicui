import { useState } from "react";
import { Checkbox, CheckboxRow } from "../components/ui/checkbox";

function CheckboxHarness({ emit, checked: initial = false, shape, ...props }: any) {
  const [checked, setChecked] = useState<boolean>(initial);
  const control = (
    <Checkbox
      {...props}
      shape={shape}
      checked={checked}
      onCheckedChange={(next: boolean) => {
        setChecked(next);
        emit("checkedChange", { checked: next });
      }}
    />
  );
  return shape === "round" ? <CheckboxRow>{control}</CheckboxRow> : control;
}

export default { id: "ui.checkbox", Component: CheckboxHarness, events: ["checkedChange"] };
