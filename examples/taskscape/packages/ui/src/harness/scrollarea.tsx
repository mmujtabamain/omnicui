import { ScrollArea } from "../components/ui/scroll-area";

const TASKS = Array.from({ length: 16 }, (_, i) => `task-${i + 1}`);

function ScrollAreaHarness({ emit, ...props }: any) {
  const horizontal = props.orientation === "horizontal";
  return (
    <div className="h-screen w-screen">
      <ScrollArea
        {...props}
        onScrollingChanged={(scrolling: boolean) => emit("scrollingChanged", { scrolling })}
        onBottomChanged={(atBottom: boolean) => emit("bottomChanged", { atBottom })}
        onDropOverChanged={(over: boolean) => emit("dropOverChanged", { over })}
        onRootDropped={(taskId: string) => emit("rootDropped", { taskId })}
        onPressBlank={() => emit("pressBlank", { clearedSelection: true })}
      >
        {TASKS.map((id, i) => (
          <div
            key={id}
            data-task-id={id}
            draggable
            data-slot={i === 0 ? "firstRow" : i === TASKS.length - 1 ? "lastRow" : undefined}
            className={
              horizontal
                ? "flex h-[13rem] w-[10rem] shrink-0 items-start border-r border-edge-1 p-3"
                : "flex h-[2.5rem] shrink-0 items-center border-b border-edge-1 px-3"
            }
          >
            Task {i + 1}
          </div>
        ))}
      </ScrollArea>
    </div>
  );
}

export default {
  id: "ui.scrollarea",
  Component: ScrollAreaHarness,
  events: ["scrollingChanged", "bottomChanged", "dropOverChanged", "rootDropped", "pressBlank"],
};
