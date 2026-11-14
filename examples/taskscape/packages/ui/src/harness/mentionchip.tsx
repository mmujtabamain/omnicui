import { useState } from "react";
import { MentionChip } from "../components/ui/mention-chip";

function MentionChipHarness({ emit, source = "local", name = "attachment", ...props }: any) {
  const [gone, setGone] = useState(false);
  if (gone) return null;
  return (
    <MentionChip
      {...props}
      source={source}
      name={name}
      onOpen={() => emit("opened", { source, name })}
      onRemove={() => {
        setGone(true);
        emit("removed", { name });
      }}
    />
  );
}

export default { id: "ui.mentionchip", Component: MentionChipHarness, events: ["opened", "removed"] };
