import { useState } from "react";
import { MentionChip } from "../components/ui/mention-chip";

function MentionChipHarness({ emit, source = "local", name = "attachment", ...props }: any) {
  const [removed, setRemoved] = useState(false);
  return (
    <MentionChip
      {...props}
      hidden={removed || undefined}
      source={source}
      name={name}
      onOpen={() => emit("opened", { source, name })}
      onRemove={() => {
        setRemoved(true);
        emit("removed", { name });
      }}
    />
  );
}

export default { id: "ui.mentionchip", Component: MentionChipHarness, events: ["opened", "removed"] };
