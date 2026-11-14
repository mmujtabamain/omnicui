import { useState } from "react";
import { Resizer } from "../components/ui/resizer";

const DEFAULT_SPLIT = 0.5;

function DemoPane({ title, lines }: { title: string; lines: string[] }) {
  return (
    <div className="flex min-w-0 flex-col gap-1 p-2">
      <div className="truncate text-[0.929rem] font-medium text-content-1">{title}</div>
      {lines.map((line) => (
        <div key={line} className="truncate text-[0.857rem] text-content-3">{line}</div>
      ))}
    </div>
  );
}

function ResizerHarness({ emit, value, min, max, reset, open, label, orientation }: any) {
  const split = reset === "double-click";
  const [current, setCurrent] = useState<number>(value ?? DEFAULT_SPLIT);
  const showAside = split || open !== false;

  return (
    <div className="flex h-[10rem] w-[40rem] overflow-hidden rounded-(--radius-panel) border border-edge-1 bg-surface-1">
      <div
        className="flex min-w-0 flex-1 flex-col bg-surface-2"
        style={split ? { flex: "0 0 auto", width: `${current * 100}%` } : undefined}
      >
        <DemoPane title="Tasks" lines={["Plan the sprint", "Review onboarding", "Ship the export"]} />
      </div>
      <Resizer
        orientation={orientation}
        value={current}
        min={min}
        max={max}
        reset={reset}
        open={open}
        label={label}
        onResize={(v: number) => {
          setCurrent(v);
          emit("resized", { value: v });
        }}
        onReset={() => {
          setCurrent(DEFAULT_SPLIT);
          emit("reset", {});
        }}
      />
      {showAside && (
        <div
          className="flex min-w-0 flex-1 flex-col bg-surface-2"
          style={split ? undefined : { flex: "0 0 auto", width: `${current}px` }}
        >
          <DemoPane title={split ? "Details" : "Preview"} lines={["Updated 2 hours ago", "Owned by Ana"]} />
        </div>
      )}
    </div>
  );
}

export default { id: "ui.resizer", Component: ResizerHarness, events: ["resized", "reset"] };
