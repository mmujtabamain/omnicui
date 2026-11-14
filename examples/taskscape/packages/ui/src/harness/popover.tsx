import { useState } from "react";
import { Popover, PopoverItem } from "../components/ui/popover";

const PEOPLE = ["alice", "albert", "alina", "alan", "alexis", "alfred", "allison", "bruno"];

function PopoverHarness({ emit, maxItems = 6, anchorX = 0, anchorY = 0 }: any) {
  const [text, setText] = useState("");
  const [dismissed, setDismissed] = useState(false);
  const query = text.slice(1).toLowerCase();
  const matches =
    text.startsWith("@") && !text.includes(" ") ? PEOPLE.filter((p) => p.includes(query)) : [];
  const open = !dismissed && matches.length > 0;

  return (
    <div className="flex h-[34rem] w-full flex-col justify-end bg-surface-1 p-4 text-content-1">
      <div className="w-[16rem]">
        <Popover
          value={text}
          onValueChange={(v) => {
            setDismissed(false);
            setText(v);
          }}
          open={open}
          anchor={{ x: anchorX, y: anchorY }}
          maxItems={maxItems}
          placeholder="Type @ to mention"
          onEscape={() => {
            setDismissed(true);
            emit("escaped", {});
          }}
          onPick={(value: string) => {
            setDismissed(true);
            setText(`${value} `);
            emit("picked", { value });
          }}
        >
          {matches.map((name) => (
            <PopoverItem key={name} value={`@${name}`} />
          ))}
        </Popover>
      </div>
    </div>
  );
}

export default { id: "ui.popover", Component: PopoverHarness, events: ["picked", "escaped"] };
