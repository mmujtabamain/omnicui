import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * The day-count field: a native number input with its spin buttons hidden, centred figures and a
 * fixed compact size (1rem = 14px: h-6.5 is 22.75px, w-12 is 42px). There is no size prop and no
 * variant. Focus shows a 1px ring in the neutral foreground token, not the accent.
 */
export const numberFieldVariants = cva([
  "block h-6.5 w-12 min-w-0 rounded-(--radius-field) border border-edge-2 bg-surface-3 px-1",
  "text-center text-[0.929rem] tabular-nums text-content-1 outline-none",
  "transition-[color,border-color,box-shadow] duration-(--duration-fast) ease-standard",
  "[appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none",
  "hover:border-edge-3 focus-visible:ring-1 focus-visible:ring-content-1",
  "disabled:pointer-events-none disabled:opacity-45",
]);

export interface NumberFieldProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "type" | "value" | "defaultValue" | "min" | "max" | "step"> {
  /** Controlled day count, shown clamped to min..max. */
  value?: number;
  /** Receives the clamped value on every change that alters it. */
  onValueChange?: (value: number) => void;
  /** Lowest day count. Default 1. */
  min?: number;
  /** Highest day count. Default 365. */
  max?: number;
  /** Accessible name. Required by the spec: the day count has no visible label. */
  ariaLabel?: string;
}

const clamp = (n: number, min: number, max: number) => Math.min(max, Math.max(min, n));

/** Whole days typed in the field; null while the field is empty or not a number. */
const parseDays = (text: string): number | null => {
  if (text.trim() === "") return null;
  const n = Number(text);
  return Number.isFinite(n) ? Math.trunc(n) : null;
};

export const NumberField = React.forwardRef<HTMLInputElement, NumberFieldProps>(function NumberField(
  { value = 1, onValueChange, min = 1, max = 365, ariaLabel = "", className, onChange, onBlur, onKeyDown, ...props },
  ref,
) {
  const inputRef = React.useRef<HTMLInputElement>(null);
  React.useImperativeHandle(ref, () => inputRef.current as HTMLInputElement);

  // The displayed text and the last day count emitted (or seeded from the value prop) live here,
  // so an edit shows at once. A new value prop replaces both.
  const [seen, setSeen] = React.useState(value);
  const [committed, setCommitted] = React.useState(() => clamp(value, min, max));
  const [text, setText] = React.useState(() => String(clamp(value, min, max)));
  if (value !== seen) {
    setSeen(value);
    setCommitted(clamp(value, min, max));
    setText(String(clamp(value, min, max)));
  }

  const commit = (next: number) => {
    if (next === committed) return;
    setCommitted(next);
    onValueChange?.(next);
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange?.(e);
    const raw = e.target.value;
    const days = parseDays(raw);
    if (days === null) {
      setText(raw);
      return;
    }
    if (days > max) {
      setText(String(max));
      commit(max);
      return;
    }
    setText(raw);
    // Below min the typed value is shown and held until blur: more digits may bring it into range.
    if (days >= min) commit(days);
  };

  const handleBlur = (e: React.FocusEvent<HTMLInputElement>) => {
    onBlur?.(e);
    const days = parseDays(text);
    const next = days === null ? committed : clamp(days, min, max);
    setText(String(next));
    commit(next);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    onKeyDown?.(e);
    const current = parseDays(text) ?? committed;
    let next: number;
    switch (e.key) {
      case "ArrowUp":
        next = clamp(current + 1, min, max);
        break;
      case "ArrowDown":
        next = clamp(current - 1, min, max);
        break;
      case "Home":
        next = min;
        break;
      case "End":
        next = max;
        break;
      default:
        return;
    }
    e.preventDefault();
    setText(String(next));
    commit(next);
  };

  // The mouse wheel does not step a focused field (native number inputs do). React's wheel listener
  // is passive, so the native one is attached here to be able to cancel it.
  React.useEffect(() => {
    const el = inputRef.current;
    if (!el) return;
    const stopWheel = (e: WheelEvent) => {
      if (document.activeElement === el) e.preventDefault();
    };
    el.addEventListener("wheel", stopWheel, { passive: false });
    return () => el.removeEventListener("wheel", stopWheel);
  }, []);

  return (
    <input
      {...props}
      ref={inputRef}
      data-slot="field"
      type="number"
      inputMode="numeric"
      min={min}
      max={max}
      step={1}
      aria-label={ariaLabel || undefined}
      aria-valuemin={min}
      aria-valuemax={max}
      value={text}
      onChange={handleChange}
      onBlur={handleBlur}
      onKeyDown={handleKeyDown}
      className={cn(numberFieldVariants(), className)}
    />
  );
});
