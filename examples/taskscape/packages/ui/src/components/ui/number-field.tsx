import * as React from "react";
import { cn } from "../../lib/utils";

/** Whole days: a fraction is truncated, then the value is pinned to min..max. */
function clampDays(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, Math.trunc(value)));
}

const FIELD = [
  "box-border block h-6.5 w-12 min-w-0 rounded-(--radius-field) border border-edge-2 bg-surface-3 px-0 py-0",
  "text-center text-[0.929rem] leading-none tabular-nums text-content-1 caret-content-1",
  "outline-none transition-[border-color,box-shadow] duration-(--duration-fast) ease-standard",
  "hover:border-edge-3 focus:border-edge-3 focus:ring-1 focus:ring-content-1",
  "[appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none",
].join(" ");

export interface NumberFieldProps
  extends Omit<
    React.InputHTMLAttributes<HTMLInputElement>,
    "type" | "value" | "defaultValue" | "onChange" | "min" | "max"
  > {
  /** Controlled day count. Shown clamped to min..max; a value outside the range is not emitted. */
  value?: number;
  /** Lowest day count. Typed values below it are held until blur. */
  min?: number;
  /** Highest day count. Values above it are pinned to it. */
  max?: number;
  /** Required by APG: the day count has no visible label of its own. */
  ariaLabel?: string;
  /** Receives the clamped value on every user change that alters it. */
  onValueChange?: (value: number) => void;
}

/**
 * Narrow day-count input: a spinbutton with the native spin buttons hidden. It keeps the text it
 * shows in its own state, seeded from `value`, and re-seeds when the `value` prop changes.
 */
export const NumberField = React.forwardRef<HTMLInputElement, NumberFieldProps>(function NumberField(
  { value = 1, min = 1, max = 365, ariaLabel = "", onValueChange, onKeyDown, onBlur, className, ...props },
  ref,
) {
  const [committed, setCommitted] = React.useState(() => clampDays(value, min, max));
  const [shown, setShown] = React.useState(() => String(clampDays(value, min, max)));
  const inner = React.useRef<HTMLInputElement | null>(null);

  const setRef = React.useCallback(
    (node: HTMLInputElement | null) => {
      inner.current = node;
      if (typeof ref === "function") ref(node);
      else if (ref) ref.current = node;
    },
    [ref],
  );

  // A changed value prop replaces the shown text: the host echoes emitted changes back.
  React.useEffect(() => {
    const next = clampDays(value, min, max);
    setCommitted(next);
    setShown(String(next));
  }, [value, min, max]);

  // The mouse wheel must not step a focused field. React's wheel listener is passive, so the
  // native listener is attached here to be able to cancel it.
  React.useEffect(() => {
    const node = inner.current;
    if (!node) return;
    const stopWheel = (e: WheelEvent) => {
      if (document.activeElement === node) e.preventDefault();
    };
    node.addEventListener("wheel", stopWheel, { passive: false });
    return () => node.removeEventListener("wheel", stopWheel);
  }, []);

  /** Shows `display`, and emits `next` only when it differs from the last emitted value. */
  const commit = (next: number, display: string = String(next)) => {
    setShown(display);
    if (next !== committed) {
      setCommitted(next);
      onValueChange?.(next);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const raw = e.target.value;
    if (raw === "") {
      setShown("");
      return;
    }
    const n = Number(raw);
    if (!Number.isFinite(n)) {
      setShown(raw);
      return;
    }
    const whole = Math.trunc(n);
    if (whole > max) {
      commit(max);
      return;
    }
    if (whole < min) {
      // Held as typed: more digits may bring it into range. Clamped on blur.
      setShown(raw);
      return;
    }
    commit(whole, Number.isInteger(n) ? raw : String(whole));
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    onKeyDown?.(e);
    if (e.defaultPrevented) return;
    const parsed = shown === "" ? NaN : Number(shown);
    switch (e.key) {
      case "ArrowUp":
      case "ArrowDown": {
        e.preventDefault();
        const current = Number.isFinite(parsed) ? Math.trunc(parsed) : committed;
        commit(clampDays(current + (e.key === "ArrowUp" ? 1 : -1), min, max));
        return;
      }
      case "Home":
        e.preventDefault();
        commit(min);
        return;
      case "End":
        e.preventDefault();
        commit(max);
        return;
    }
  };

  const handleBlur = (e: React.FocusEvent<HTMLInputElement>) => {
    onBlur?.(e);
    const n = shown === "" ? NaN : Number(shown);
    // An empty field reverts to the last emitted value; anything else is clamped.
    if (Number.isFinite(n)) commit(clampDays(n, min, max));
    else setShown(String(committed));
  };

  return (
    <input
      {...props}
      ref={setRef}
      data-slot="field"
      type="number"
      min={min}
      max={max}
      step={1}
      aria-label={ariaLabel || undefined}
      aria-valuemin={min}
      aria-valuemax={max}
      value={shown}
      onChange={handleChange}
      onKeyDown={handleKeyDown}
      onBlur={handleBlur}
      className={cn(FIELD, className)}
    />
  );
});
