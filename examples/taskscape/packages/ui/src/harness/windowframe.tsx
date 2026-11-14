import { useRef } from "react";
import type { KeyboardEvent } from "react";
import {
  WindowFrame,
  WindowFrameBody,
  WindowFrameCaptureNotes,
  WindowFrameCaptureTitle,
  WindowFrameOverlays,
  WindowFrameTitleBar,
} from "../components/ui/window-frame";

/** The capture bar: Escape dismisses from any field, Enter and Mod+Enter submit, Tab toggles title and notes. */
function CaptureBar({ emit, ...props }: any) {
  const title = useRef<HTMLInputElement>(null);
  const notes = useRef<HTMLTextAreaElement>(null);

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Escape") {
      e.preventDefault();
      emit("dismissed", { reason: "escape" });
    } else if (e.key === "Enter") {
      if (e.metaKey || e.ctrlKey) {
        e.preventDefault();
        emit("submitted", { via: "shortcut" });
      } else if (e.target === title.current) {
        e.preventDefault();
        emit("submitted", { via: "enter" });
      }
    } else if (e.key === "Tab") {
      e.preventDefault();
      (e.target === title.current ? notes.current : title.current)?.focus();
    }
  };

  return (
    <WindowFrame {...props} chrome="frameless" onKeyDown={onKeyDown}>
      <WindowFrameCaptureTitle ref={title} placeholder="What needs doing?" />
      <WindowFrameCaptureNotes ref={notes} placeholder="Notes" />
    </WindowFrame>
  );
}

/** The full app window: a title bar above the body, with demo content. */
function TitledFrame(props: any) {
  return (
    <WindowFrame {...props} chrome="titled">
      <WindowFrameTitleBar className="h-[2.5rem] border-b border-edge-1 px-3 text-[0.929rem] font-medium">
        Taskscape
      </WindowFrameTitleBar>
      <WindowFrameBody>
        <div className="w-[11rem] shrink-0 border-r border-edge-1 bg-surface-0 p-2 text-[0.929rem] text-content-2">
          Projects
        </div>
        <div className="min-w-0 flex-1 p-3 text-[0.929rem] text-content-1">Pane content</div>
      </WindowFrameBody>
      <WindowFrameOverlays />
    </WindowFrame>
  );
}

function WindowFrameHarness({ emit, chrome = "titled", ...props }: any) {
  if (chrome === "frameless") return <CaptureBar emit={emit} {...props} />;
  return <TitledFrame {...props} />;
}

export default {
  id: "ui.windowframe",
  Component: WindowFrameHarness,
  events: ["dismissed", "submitted"],
};
