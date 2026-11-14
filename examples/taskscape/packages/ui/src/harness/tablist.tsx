import { useRef, useState } from "react";
import { TabList, TabListItem, tabSlot } from "../components/ui/tab-list";

interface TabListHarnessProps {
  emit: (name: string, fields: { value: string }) => void;
  value?: string;
  loop?: boolean;
  label?: string;
  orientation?: "horizontal" | "vertical";
  overflow?: "scroll" | "fit";
  tabCount?: number;
}

function TabListHarness({
  emit,
  value = "1",
  loop = true,
  label = "Tabs",
  orientation = "horizontal",
  overflow = "scroll",
  tabCount = 3,
}: TabListHarnessProps) {
  const [selected, setSelected] = useState(value);
  const current = useRef(value);
  const change = (next: string) => {
    if (next === current.current) return;
    current.current = next;
    setSelected(next);
    emit("valueChanged", { value: next });
  };
  const frame =
    orientation === "horizontal"
      ? "flex h-[2.5rem] w-[24rem] items-stretch border-b border-edge-1 bg-surface-1 px-1"
      : "flex w-[12rem] flex-col bg-surface-1 p-2";
  return (
    <div className={frame}>
      <TabList
        value={selected}
        onValueChange={change}
        loop={loop}
        label={label}
        orientation={orientation}
        overflow={overflow}
      >
        {Array.from({ length: tabCount }, (_, i) => {
          const id = String(i + 1);
          return (
            <TabListItem key={id} value={id} data-slot={tabSlot(i, tabCount)}>
              {`List ${id}`}
            </TabListItem>
          );
        })}
      </TabList>
    </div>
  );
}

export default { id: "ui.tablist", Component: TabListHarness, events: ["valueChanged"] };
