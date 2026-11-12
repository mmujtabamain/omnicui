import type { Filter } from "../model.ts";
import { Parts } from "../omnicui/list.ts";

const FILTERS: { value: Filter; label: string; part: string }[] = [
  { value: "all", label: "All", part: Parts.filterAll },
  { value: "active", label: "Active", part: Parts.filterActive },
  { value: "done", label: "Done", part: Parts.filterDone },
];

/** All, Active, Done: a segmented control of toggle buttons (aria-pressed), one chosen. */
export function FilterBar({ filter, onFilter }: { filter: Filter; onFilter: (f: Filter) => void }) {
  return (
    <div className="segmented" role="group" aria-label="Show">
      {FILTERS.map((f) => (
        <button key={f.value} type="button" className="segment" aria-pressed={f.value === filter} data-part={f.part} onClick={() => onFilter(f.value)}>
          {f.label}
        </button>
      ))}
    </div>
  );
}
