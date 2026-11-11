import { useState } from "react";
import { CounterDefaults, Parts, type CounterProps } from "../../omnicui/counter.ts";
import { defect } from "../defects.ts";

// The counter, redesigned: a vertical card, the value large in a pill, a round icon button.
// Variants (OMNICUI_DEFECTS), each on top of it:
//   webalt.counter.copy       the value reads "3 taps"            (same behaviour, other copy)
//   webalt.counter.srlabel    a visually-hidden "Count: " inside the value (for screen readers)
//   webalt.counter.toolbar    a heading and a Reset button beside it, which the contract doesn't name
//   webalt.counter.double     each press adds two                 (broken behaviour)
//   webalt.counter.divbutton  the button is a <div role="button"> with no key handling (broken)
//   webalt.brand              the same layout in the original design language (its colours, its font)
export function RedesignedCounter({ start = CounterDefaults.start, onChanged }: CounterProps) {
  const [count, setCount] = useState(start);
  const step = defect("webalt.counter.double") ? 2 : 1;
  const increment = () => {
    setCount(count + step);
    onChanged?.({ count: count + step });
  };
  const plus = (
    <svg width="20" height="20" viewBox="0 0 20 20" aria-hidden="true">
      <path d="M10 3v14M3 10h14" stroke="currentColor" strokeWidth="3" strokeLinecap="round" />
    </svg>
  );
  const button = defect("webalt.counter.divbutton") ? (
    <div role="button" tabIndex={0} className="rd-counter__plus" data-part={Parts.increment} aria-label="Increment" onClick={increment}>
      {plus}
    </div>
  ) : (
    <button type="button" className="rd-counter__plus" data-part={Parts.increment} aria-label="Increment" onClick={increment}>
      {plus}
    </button>
  );
  return (
    <div className={defect("webalt.brand") ? "rd rd--brand rd-counter" : "rd rd-counter"}>
      {defect("webalt.counter.toolbar") && <h2 className="rd-counter__heading">Visitors</h2>}
      <output className="rd-counter__value" data-part={Parts.value}>
        {defect("webalt.counter.srlabel") && <span className="rd-sr">Count: </span>}
        {count}
        {defect("webalt.counter.copy") && " taps"}
      </output>
      <div className="rd-counter__row">
        {button}
        {defect("webalt.counter.toolbar") && (
          <button type="button" className="rd-counter__reset" onClick={() => setCount(start)}>
            Reset
          </button>
        )}
      </div>
    </div>
  );
}
