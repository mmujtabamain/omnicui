import { useLayoutEffect, useRef, useState } from "react";
import { Popover, PopoverEditor, PopoverItem, PopoverList } from "../components/ui/popover";

const NAMES = ["alan", "alice", "albert", "alexandra", "amara", "alina", "bartholomew", "gabriel"];

function PopoverHarness({ emit, maxItems = 6, anchorX = 0, anchorY = 0 }: any) {
  const [text, setText] = useState("");
  const [active, setActive] = useState(0);
  const [dismissed, setDismissed] = useState(false);
  const [measured, setMeasured] = useState({ x: 0, y: 0 });
  const editor = useRef<HTMLInputElement>(null);

  // Without an explicit anchor, the caret sits at the editor's bottom-left.
  useLayoutEffect(() => {
    const r = editor.current?.getBoundingClientRect();
    if (r) setMeasured({ x: r.left, y: r.bottom });
  }, []);
  const caret = anchorX || anchorY ? { x: anchorX, y: anchorY } : measured;

  const match = /@(\w*)$/.exec(text);
  const query = match ? match[1].toLowerCase() : "";
  const matches = match ? NAMES.filter((name) => name.includes(query)) : [];
  const open = matches.length > 0 && !dismissed;

  const change = (value: string) => {
    setText(value);
    setActive(0);
    setDismissed(false);
  };

  const pick = (index: number) => {
    if (!match) return;
    const value = matches[index];
    setText(text.slice(0, match.index) + `@${value} `);
    emit("picked", { value });
  };

  const dismissByEscape = () => {
    setDismissed(true);
    emit("escaped", {});
  };

  return (
    <div className="p-4">
      <Popover
        open={open}
        anchor={caret}
        maxItems={maxItems}
        matchCount={matches.length}
        active={active}
        onActiveChange={setActive}
        onPick={pick}
        onEscape={dismissByEscape}
        onOpenChange={(o) => {
          if (!o) setDismissed(true);
        }}
      >
        <PopoverEditor
          ref={editor}
          aria-label="Mention"
          placeholder="Type @ to mention"
          className="w-60"
          value={text}
          onChange={(e) => change(e.target.value)}
        />
        <PopoverList>
          {matches.map((name, i) => (
            <PopoverItem key={name} index={i} data-slot={`row${i + 1}`}>
              {name}
            </PopoverItem>
          ))}
        </PopoverList>
      </Popover>
    </div>
  );
}

export default { id: "ui.popover", Component: PopoverHarness, events: ["picked", "escaped"] };
