import { useId } from "react";

/** A switch: a <button role="switch"> named by its visible label; pressing the label flips it too. */
export function Switch({ label, checked, onChange, part }: { label: string; checked: boolean; onChange: (on: boolean) => void; part: string }) {
  const id = useId();
  return (
    <div className="switch-row">
      <label htmlFor={id} className="switch-row__label">{label}</label>
      <button id={id} type="button" role="switch" aria-checked={checked} className="switch" data-part={part} onClick={() => onChange(!checked)}>
        <span className="switch__thumb" />
      </button>
    </div>
  );
}
