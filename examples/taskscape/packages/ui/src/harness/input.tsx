import { useEffect, useState } from "react";
import { Input } from "../components/ui/input";

function InputHarness({ emit, value: initialValue = "", ...props }: any) {
  const [value, setValue] = useState<string>(initialValue);

  // Test hook: every keydown that reaches the window (i.e. was not stopped by the field).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => emit("propagated", { key: e.key });
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [emit]);

  return (
    <Input
      {...props}
      value={value}
      onValueChange={(next: string) => {
        setValue(next);
        emit("valueChange", { value: next });
      }}
      onSubmit={(next: string) => emit("submit", { value: next })}
      onCommit={(next: string) => emit("commit", { value: next })}
      onCancel={() => emit("cancel", {})}
    />
  );
}

export default {
  id: "ui.input",
  Component: InputHarness,
  events: ["valueChange", "submit", "commit", "cancel", "propagated"],
};
