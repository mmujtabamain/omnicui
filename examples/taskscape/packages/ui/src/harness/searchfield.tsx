import { useState } from "react";
import { SearchField } from "../components/ui/search-field";

function SearchFieldHarness({ emit, value = "", ...props }: any) {
  const [query, setQuery] = useState<string>(value);
  return (
    <SearchField
      {...props}
      value={query}
      onValueChange={(next: string) => {
        setQuery(next);
        emit("changed", { value: next });
      }}
      onStepMatch={(direction: string) => emit("stepMatch", { direction })}
      onClear={() => emit("cleared", {})}
    />
  );
}

export default {
  id: "ui.searchfield",
  Component: SearchFieldHarness,
  events: ["changed", "stepMatch", "cleared"],
};
