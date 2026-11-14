import { useState } from "react";
import { SettingRow } from "../components/ui/setting-row";

// The contract cannot pass a key chord, so the harness supplies a demo one for the keycap.
const DEMO_KEYS = "⌘N";

function SettingRowHarness({ emit, ...props }: any) {
  const [checked, setChecked] = useState<boolean>(props.checked ?? false);
  const [choice, setChoice] = useState<string>(props.choice ?? "");
  const [recording, setRecording] = useState<boolean>(props.recording ?? false);
  return (
    <SettingRow
      {...props}
      checked={checked}
      choice={choice}
      recording={recording}
      keys={DEMO_KEYS}
      onCheckedChange={(next: boolean) => {
        setChecked(next);
        emit("toggled", { checked: next });
      }}
      onChoiceChange={(next: string) => {
        setChoice(next);
        emit("changed", { value: next });
      }}
      onRecordingChange={setRecording}
    />
  );
}

export default { id: "ui.settingrow", Component: SettingRowHarness, events: ["changed", "toggled"] };
