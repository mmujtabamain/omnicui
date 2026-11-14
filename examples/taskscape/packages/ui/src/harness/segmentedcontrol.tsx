import { useMemo, useState } from "react";
import { SegmentedControl, type SegmentedControlOption } from "../components/ui/segmented-control";

/** "value=Label" pairs in order, with one icon name per pair in the same order. */
function parseSegments(segments: string, icons: string): SegmentedControlOption[] {
  const names = icons.split(",").map((name) => name.trim());
  return segments.split(",").map((pair, index) => {
    const at = pair.indexOf("=");
    const value = at < 0 ? pair.trim() : pair.slice(0, at).trim();
    const label = at < 0 ? value : pair.slice(at + 1).trim();
    return { value, label, icon: names[index] || undefined };
  });
}

function SegmentedControlHarness({
  emit,
  value = "all",
  segments = "all=All,active=Active,done=Done",
  icons = "",
  groupName,
  tone = "accent",
  shape = "icon",
  variant = "text",
  size = "default",
  disabled = false,
}: any) {
  const [selected, setSelected] = useState<string>(value);
  const options = useMemo(() => parseSegments(segments, icons), [segments, icons]);
  return (
    <SegmentedControl
      options={options}
      value={selected}
      onValueChange={(next) => {
        setSelected(next);
        emit("valueChanged", { value: next });
      }}
      ariaLabel={groupName}
      tone={tone}
      shape={shape}
      variant={variant}
      size={size}
      disabled={disabled}
    />
  );
}

export default { id: "ui.segmentedcontrol", Component: SegmentedControlHarness, events: ["valueChanged"] };
