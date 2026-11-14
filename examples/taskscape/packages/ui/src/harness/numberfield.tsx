import { useState } from "react";
import { NumberField } from "../components/ui/number-field";

function NumberFieldHarness({ emit, value, ...props }: any) {
  // The host keeps the day count in state, seeded from the prop, so an emitted change is shown.
  const [days, setDays] = useState<number>(value ?? 1);
  return (
    <NumberField
      {...props}
      value={days}
      onValueChange={(next: number) => {
        setDays(next);
        emit("changed", { value: next });
      }}
    />
  );
}

export default { id: "ui.numberfield", Component: NumberFieldHarness, events: ["changed"] };
