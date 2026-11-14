import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";

/** The inner input. A string tag, as Button does with its "button" element. */
const Field = "input" as const;

/**
 * The frame: border, fill, focus ring. The leading and trailing slots and the inner input
 * sit inside it. shape sets the outline, tone the fill, size the height and text size.
 */
const inputGroupVariants = cva(
  [
    "group/input-group relative flex min-w-0 items-center outline-none",
    "transition-[border-color,background-color,opacity] duration-(--duration-fast) ease-standard",
    "focus-within:ring-2 focus-within:ring-focus-1",
    "data-[disabled]:opacity-45 data-[disabled]:pointer-events-none",
  ],
  {
    variants: {
      shape: {
        well: "rounded-(--radius-field) border border-edge-2",
        bar: "rounded-none border-0 border-b border-edge-1",
      },
      tone: {
        field: "bg-surface-1",
        search: "rounded-full border-0 bg-wash-2",
      },
      size: {
        md: "h-[2.5rem] gap-2 px-2 text-[1rem]",
        sm: "h-[1.75rem] gap-1.5 px-1.5 text-[0.857rem]",
      },
    },
    defaultVariants: { shape: "well", tone: "field", size: "md" },
  },
);

type Variants = VariantProps<typeof inputGroupVariants>;

const FIELD_TEXT: Record<"md" | "sm", string> = { md: "text-[1rem]", sm: "text-[0.857rem]" };

const FIELD =
  "min-w-0 flex-1 self-stretch bg-transparent p-0 text-content-1 placeholder:text-content-3 outline-none disabled:cursor-not-allowed";
const LEADING = "inline-flex shrink-0 items-center text-content-3";
const SUFFIX = "shrink-0 select-none whitespace-pre text-content-3";
const TRAILING = "inline-flex shrink-0 items-center gap-0.5";

export interface InputGroupProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "defaultValue"> {
  /** well: rounded bordered bar. bar: full-width row with a bottom edge. */
  shape?: Variants["shape"];
  /** field: bordered, surface fill. search: filled grey pill, no border. */
  tone?: Variants["tone"];
  /** md: 2.5rem high, 14px text. sm: 1.75rem high, 12px text. */
  size?: Variants["size"];
  /** Locked, greyed tail after the typed value. "" means none. */
  suffix?: string;
  /** A press anywhere in the well focuses the inner input. */
  focusInputOnPress?: boolean;
  disabled?: boolean;
  /** Renders the roll dice button. */
  sheet?: boolean;
  /** Icon in the leading slot. */
  leading?: React.ReactNode;
  /** Controlled typed value. Without it the frame keeps its own. */
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  /** Pressing the roll dice button. */
  onRoll?: () => void;
  /** Accessible name of the inner input. */
  inputLabel?: string;
  placeholder?: string;
}

export const InputGroup = React.forwardRef<HTMLDivElement, InputGroupProps>(function InputGroup(
  {
    shape = "well", tone = "field", size = "md", suffix = "", focusInputOnPress = true,
    disabled = false, sheet = false, leading, value, defaultValue = "", onValueChange, onRoll,
    inputLabel = "Input", placeholder, className, onMouseDown, ...props
  },
  ref,
) {
  const inputRef = React.useRef<HTMLInputElement>(null);
  const [inner, setInner] = React.useState(defaultValue);
  const current = value ?? inner;
  const filled = current !== "";
  const shapeName = shape ?? "well";
  const toneName = tone ?? "field";
  const sizeName = size ?? "md";
  const buttonSize = sizeName === "md" ? "md" : "sm";
  const role = shapeName === "bar" ? "combobox" : toneName === "search" ? "searchbox" : "textbox";

  const change = (next: string) => {
    if (value === undefined) setInner(next);
    onValueChange?.(next);
  };

  function handleMouseDown(e: React.MouseEvent<HTMLDivElement>) {
    onMouseDown?.(e);
    if (!focusInputOnPress || disabled) return;
    // A press on a button or on the input keeps its own behaviour.
    if ((e.target as Element).closest("button, input")) return;
    e.preventDefault();
    inputRef.current?.focus();
  }

  function clear() {
    change("");
    inputRef.current?.focus();
  }

  return (
    <div
      ref={ref}
      data-slot="well"
      data-shape={shapeName}
      data-tone={toneName}
      data-disabled={disabled || undefined}
      onMouseDown={handleMouseDown}
      className={cn(inputGroupVariants({ shape: shapeName, tone: toneName, size: sizeName }), className)}
      {...props}
    >
      {leading && (
        <span data-slot="leading" className={LEADING}>
          {leading}
        </span>
      )}
      <Field
        ref={inputRef}
        data-slot="input"
        type="text"
        role={role}
        aria-label={inputLabel}
        autoComplete="off"
        spellCheck={false}
        disabled={disabled}
        placeholder={placeholder}
        value={current}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => change(e.target.value)}
        className={cn(FIELD, FIELD_TEXT[sizeName === "sm" ? "sm" : "md"])}
      />
      {suffix !== "" && (
        <span data-slot="suffix" className={SUFFIX}>
          {suffix}
        </span>
      )}
      {(filled || sheet) && (
        <span data-slot="trailing" className={TRAILING}>
          {filled && (
            <Button
              data-slot="clear"
              ariaLabel="Clear"
              icon="close"
              shape="icon"
              size={buttonSize}
              tone="muted"
              disabled={disabled}
              onClick={clear}
            />
          )}
          {sheet && (
            <Button
              data-slot="roll"
              ariaLabel="Roll dice"
              icon="casino"
              shape="icon"
              size={buttonSize}
              tone="muted"
              disabled={disabled}
              onClick={onRoll}
            />
          )}
        </span>
      )}
    </div>
  );
});
