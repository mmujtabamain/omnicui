import { useState } from "react";
import { CommandInput } from "../components/ui/command-input";

function CommandInputHarness({ emit, value, ...props }: any) {
  const [query, setQuery] = useState<string>(value ?? "");
  return (
    <CommandInput
      {...props}
      value={query}
      onValueChange={(next: string) => {
        setQuery(next);
        emit("queryChanged", { value: next });
      }}
      onMoveActive={(direction: "up" | "down") => emit("moveActive", { direction })}
      onSubmit={(submitted: string) => emit("submit", { value: submitted })}
    />
  );
}

export default {
  id: "ui.commandinput",
  Component: CommandInputHarness,
  events: ["queryChanged", "moveActive", "submit"],
};
