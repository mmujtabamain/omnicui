import { useId, useState } from "react";
import { Parts, SwitchDefaults, type SwitchProps } from "../omnicui/switch.ts";
import { defect } from "./defects.ts";

// The switch: a <button role="switch"> named by its visible label. The thumb slides with a CSS
// transition (200ms ease-out), which is what the contract's trace checks.
export function Switch({ on: initial = SwitchDefaults.on, label = SwitchDefaults.label, onToggled }: SwitchProps) {
  const [on, setOn] = useState(initial);
  const labelId = useId();
  const toggle = () => {
    setOn(!on);
    onToggled?.({ on: !on });
  };
  return (
    <div className={defect("web.switch.slow") ? "switch switch--slow" : "switch"}>
      <span id={labelId} className="switch__label">{label}</span>
      <button type="button" role="switch" aria-checked={on} aria-labelledby={defect("web.switch.no-name") ? undefined : labelId} data-part={Parts.switch} onClick={toggle} className="switch__track">
        <span className="switch__thumb" data-part={Parts.thumb} />
      </button>
    </div>
  );
}
