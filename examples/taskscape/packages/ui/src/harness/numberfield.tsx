import { useState } from "react";
import { NumberField } from "../components/ui/number-field";

function NumberFieldHarness({ emit, value, ...props }: any) {
  const [day, setDay] = useState<number>(value);
  return (
    <NumberField
      {...props}
      value={day}
      onValueChange={(next: number) => {
        setDay(next);
        emit("changed", { value: next });
      }}
    />
  );
}

export default { id: "ui.numberfield", Component: NumberFieldHarness, events: ["changed"] };
