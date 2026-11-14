import { useRef, useState } from "react";
import { Input } from "../components/ui/input";

function InputHarness({ emit, value, ...props }: any) {
  const [text, setText] = useState<string>(value ?? "");
  const saved = useRef<string>(value ?? "");
  return (
    <div onKeyDown={(e) => emit("propagated", { key: e.key })}>
      <Input
        {...props}
        value={text}
        onValueChange={(next: string) => {
          setText(next);
          emit("valueChange", { value: next });
        }}
        onSubmit={(next: string) => emit("submit", { value: next })}
        onCommit={(next: string) => {
          saved.current = next;
          setText(next);
          emit("commit", { value: next });
        }}
        onCancel={() => {
          setText(saved.current);
          emit("cancel", {});
        }}
      />
    </div>
  );
}

export default {
  id: "ui.input",
  Component: InputHarness,
  events: ["valueChange", "submit", "commit", "cancel", "propagated"],
};
