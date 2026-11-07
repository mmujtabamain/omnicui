import { useState } from "react";
import { CounterDefaults, Parts, type CounterProps } from "../omnicui/counter.ts";
import { defect } from "./defects.ts";

// The counter, written the React way: state in useState, the event through a callback prop.
// A real <button> gets focus, Enter, Space and its accessible name from the browser.
export function Counter({ start = CounterDefaults.start, onChanged }: CounterProps) {
  const [count, setCount] = useState(start);
  const increment = () => {
    setCount(count + 1);
    onChanged?.({ count: count + 1 });
  };
  return (
    <div className={defect("web.counter.padding") ? "counter counter--wide" : "counter"}>
      <output data-part={Parts.value}>{count}</output>
      <button type="button" data-part={Parts.increment} aria-label="Increment" onClick={increment}>
        +
      </button>
    </div>
  );
}
