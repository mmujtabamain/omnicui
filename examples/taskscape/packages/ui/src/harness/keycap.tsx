import { useState } from "react";
import { Keycap } from "../components/ui/keycap";

function KeycapHarness({ emit, mode: seededMode, ...props }: any) {
  // The owner holds the editor state: recording starts on activation and ends on cancel.
  const [mode, setMode] = useState<string>(seededMode ?? "idle");
  return (
    <Keycap
      {...props}
      mode={mode as "idle" | "recording"}
      onStartRecording={() => {
        setMode("recording");
        emit("startRecording", {});
      }}
      onCapture={(accel: string | null) => emit("capture", { accel: accel ?? "" })}
      onCancel={() => {
        setMode("idle");
        emit("cancel", {});
      }}
      onReset={() => emit("reset", {})}
      onRefused={(accel: string) => emit("refused", { accel })}
    />
  );
}

export default {
  id: "ui.keycap",
  Component: KeycapHarness,
  events: ["startRecording", "capture", "cancel", "reset", "refused"],
};
