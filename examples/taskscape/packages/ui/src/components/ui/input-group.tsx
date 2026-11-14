import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";

/**
 * The frame around one inner input: border, fill, focus ring and the leading, suffix and
 * trailing slots. The frame emits no events; the inner input owns the value and the role.
 */
export const inputGroupVariants = cva(
  "group/input-group relative flex w-full min-w-0 items-center text-content-1 outline-none transition-[border-color,background-color,box-shadow,opacity] duration-(--duration-fast) ease-standard focus-within:ring-2 focus-within:ring-focus-1 data-[disabled]:pointer-events-none data-[disabled]:opacity-45",
  {
    variants: {
      shape: {
        well: "",
        bar: "rounded-none border-b border-edge-1",
      },
      tone: {
        field: "",
        search: "",
      },
      size: {
        sm: "h-[1.75rem] px-2",
        md: "h-[2.5rem] px-2.5",
      },
    },
    compoundVariants: [
      { shape: "well", tone: "field", className: "rounded-(--radius-field) border border-edge-2 bg-surface-1" },
      { shape: "well", tone: "search", className: "rounded-full bg-wash-2" },
    ],
    defaultVariants: { shape: "well", tone: "field", size: "md" },
  },
);

/** Text size inside the well: the inner input and the suffix share it. */
export const inputGroupTextVariants = cva("", {
  variants: {
    size: {
      sm: "text-[0.857rem]",
      md: "text-[1rem]",
    },
  },
  defaultVariants: { size: "md" },
});

type InputGroupVariantProps = VariantProps<typeof inputGroupVariants>;
type InputGroupSize = NonNullable<InputGroupVariantProps["size"]>;
type InputGroupTone = NonNullable<InputGroupVariantProps["tone"]>;

export interface InputGroupProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "defaultValue"> {
  /** well: rounded bordered bar with a leading slot. bar: full-width row with a bottom edge. */
  shape?: InputGroupVariantProps["shape"];
  /** field: bordered sheet well. search: filled grey pill with no border. */
  tone?: InputGroupTone;
  /** md is 2.5rem high, sm is 1.75rem high. */
  size?: InputGroupSize;
  /** Locked, greyed tail after the typed value. The frame does not change the typed value. */
  suffix?: string;
  /** A press anywhere in the well focuses the inner input. */
  focusInputOnPress?: boolean;
  disabled?: boolean;
  /** Renders the roll dice button (sheets only). */
  sheet?: boolean;
  /** Icon in the leading slot. */
  leading?: React.ReactNode;
  /** Controlled value of the inner input. */
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  /** Pressed on the roll dice button. */
  onRoll?: () => void;
  /** The inner input, the one InputGroupInput. */
  children?: React.ReactNode;
}

type InputGroupContextValue = {
  value: string;
  setValue: (next: string) => void;
  inputRef: React.RefObject<HTMLInputElement | null>;
  size: InputGroupSize;
  tone: InputGroupTone;
  disabled: boolean;
  hasSuffix: boolean;
};

const InputGroupContext = React.createContext<InputGroupContextValue | null>(null);

function useInputGroup(): InputGroupContextValue {
  const ctx = React.useContext(InputGroupContext);
  if (!ctx) throw new Error("InputGroupInput must be rendered inside InputGroup");
  return ctx;
}

export const InputGroup = React.forwardRef<HTMLDivElement, InputGroupProps>(function InputGroup(
  {
    shape = "well", tone = "field", size = "md", suffix = "", focusInputOnPress = true, disabled = false, sheet = false,
    leading, value: valueProp, defaultValue = "", onValueChange, onRoll, className, children, onMouseDown, ...props
  },
  ref,
) {
  const inputRef = React.useRef<HTMLInputElement | null>(null);
  const [inner, setInner] = React.useState(defaultValue);
  const controlled = valueProp !== undefined;
  const value = controlled ? valueProp : inner;

  const setValue = React.useCallback(
    (next: string) => {
      if (!controlled) setInner(next);
      onValueChange?.(next);
    },
    [controlled, onValueChange],
  );

  const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    onMouseDown?.(e);
    if (!focusInputOnPress || disabled || e.defaultPrevented) return;
    if (e.target instanceof Element && e.target.closest("button, input")) return;
    e.preventDefault();
    inputRef.current?.focus();
  };

  const clear = () => {
    setValue("");
    inputRef.current?.focus();
  };

  const hasSuffix = suffix !== "";
  const hasTrailing = value !== "" || sheet;

  return (
    <InputGroupContext.Provider value={{ value, setValue, inputRef, size, tone, disabled, hasSuffix }}>
      <div
        ref={ref}
        data-slot="well"
        data-disabled={disabled || undefined}
        className={cn(inputGroupVariants({ shape, tone, size }), className)}
        onMouseDown={handleMouseDown}
        {...props}
      >
        {leading && (
          <span data-slot="leading" aria-hidden="true" className="mr-1.5 flex shrink-0 items-center">
            {leading}
          </span>
        )}
        {children}
        {hasSuffix && (
          <>
            <span data-slot="suffix" className={cn("shrink-0 select-none whitespace-pre text-content-3", inputGroupTextVariants({ size }))}>
              {suffix}
            </span>
            <span aria-hidden="true" className="min-w-0 flex-1" />
          </>
        )}
        {hasTrailing && (
          <span data-slot="trailing" className="ml-1 flex shrink-0 items-center gap-0.5">
            {value !== "" && (
              <Button
                data-slot="clear"
                size="sm"
                shape="icon"
                variant="ghost"
                tone="muted"
                icon="close"
                ariaLabel="Clear"
                description="Clear"
                disabled={disabled}
                onClick={clear}
              />
            )}
            {sheet && (
              <Button
                data-slot="roll"
                size="sm"
                shape="icon"
                variant="ghost"
                tone="muted"
                icon="casino"
                ariaLabel="Roll dice"
                description="Roll dice"
                disabled={disabled}
                onClick={onRoll}
              />
            )}
          </span>
        )}
      </div>
    </InputGroupContext.Provider>
  );
});

export interface InputGroupInputProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "value" | "defaultValue" | "type"> {}

/** The one inner input. Declared textbox; a searchbox on the search tone. */
export function InputGroupInput({ className, onChange, disabled, ...props }: InputGroupInputProps) {
  const ctx = useInputGroup();
  return (
    <input
      {...props}
      ref={ctx.inputRef}
      data-slot="input"
      type={ctx.tone === "search" ? "search" : "text"}
      value={ctx.value}
      disabled={ctx.disabled || disabled}
      onChange={(e) => {
        ctx.setValue(e.target.value);
        onChange?.(e);
      }}
      className={cn(
        inputGroupTextVariants({ size: ctx.size }),
        "h-full min-w-0 bg-transparent p-0 text-content-1 outline-none placeholder:text-content-3 disabled:cursor-not-allowed [&::-webkit-search-cancel-button]:appearance-none",
        ctx.hasSuffix ? "max-w-full flex-none [field-sizing:content]" : "flex-1",
        className,
      )}
    />
  );
}
