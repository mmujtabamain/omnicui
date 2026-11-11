import { useId, useState, type KeyboardEvent } from "react";
import { Parts, SwitchDefaults, type SwitchProps } from "../../omnicui/switch.ts";
import { defect } from "../defects.ts";

// The switch, redesigned: a native checkbox (role="switch") with its label beside it, in a dashed
// box. No track, no thumb, no animation. Variants (OMNICUI_DEFECTS), each on top of it:
//   webalt.switch.pill     an upright pill switch instead, its thumb sliding UP over 400 ms linearly
//                          (so it does have a thumb, moving another way)
//   webalt.switch.noevent  toggling doesn't send `toggled`           (broken behaviour)
//   webalt.switch.nospace  Space doesn't toggle it                   (broken behaviour)
export function RedesignedSwitch({ on: initial = SwitchDefaults.on, label = SwitchDefaults.label, onToggled }: SwitchProps) {
  const [on, setOn] = useState(initial);
  const id = useId();
  const toggle = () => {
    setOn(!on);
    if (!defect("webalt.switch.noevent")) onToggled?.({ on: !on });
  };
  const noSpace = (e: KeyboardEvent) => {
    if (defect("webalt.switch.nospace") && e.key === " ") e.preventDefault();
  };
  if (defect("webalt.switch.pill")) {
    return (
      <div className="rd rd-switch">
        <button type="button" role="switch" aria-checked={on} aria-labelledby={id} data-part={Parts.switch} className="rd-pill"
          onClick={toggle} onKeyDown={noSpace} onKeyUp={noSpace}>
          <span className="rd-pill__thumb" data-part={Parts.thumb} />
        </button>
        <span id={id}>{label}</span>
      </div>
    );
  }
  return (
    <div className="rd rd-switch">
      <input id={id} type="checkbox" role="switch" className="rd-switch__box" data-part={Parts.switch} checked={on}
        onChange={toggle} onKeyDown={noSpace} onKeyUp={noSpace} />
      <label htmlFor={id}>{label}</label>
    </div>
  );
}
