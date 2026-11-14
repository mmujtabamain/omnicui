import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/** Size sets the type and the line box. Fill is the resting surface (draft, or inline while editing). */
const inputVariants = cva(
  [
    "min-w-0 w-full rounded-(--radius-field) text-content-1 outline-none",
    "placeholder:text-content-3 selection:bg-selection-1",
    "focus:ring-focus-1 transition-[background-color,box-shadow] duration-(--duration-fast) ease-standard",
    "disabled:pointer-events-none disabled:opacity-45",
  ],
  {
    variants: {
      size: {
        inline: "h-[1.5rem] max-w-[10rem] px-space-4 text-[1rem] leading-[1.5rem] font-medium focus:ring-1",
        md: "h-[1.5rem] px-space-4 text-[1rem] leading-[1.5rem] focus:ring-2",
        sm: "h-[1.2857rem] px-space-4 text-[0.857rem] leading-[1.2857rem] focus:ring-2",
        display: "h-[1.5rem] px-space-4 text-[1.2857rem] leading-[1.5rem] font-semibold focus:ring-2",
      },
      fill: {
        none: "bg-transparent",
        resting: "bg-surface-0",
      },
      multiline: {
        false: "",
        true: "field-sizing-content h-auto min-h-[1.5rem] max-h-[3rem] resize-none overflow-y-auto py-0",
      },
    },
    defaultVariants: { size: "md", fill: "none", multiline: false },
  },
);

type Element = HTMLInputElement | HTMLTextAreaElement;
type Size = VariantProps<typeof inputVariants>["size"];

export type InputMode = "inline" | "draft" | "bare";

export interface InputProps
  extends Omit<
    React.InputHTMLAttributes<Element>,
    "size" | "value" | "defaultValue" | "onChange" | "onKeyDown" | "onFocus" | "onBlur" | "onMouseDown" | "onMouseUp" | "onSubmit"
  > {
  value?: string;
  size?: Size;
  /** inline: readonly at rest, editing on click or editRequest. draft: always editable, resting fill. bare: always editable, no fill. */
  mode?: InputMode;
  /** A textarea that grows from one line to two, then scrolls. Enter never inserts a line break. */
  multiline?: boolean;
  /** Accessible name. */
  ariaLabel?: string;
  /** Select all on keyboard focus with a non-empty value. */
  selectOnFocus?: boolean;
  /** Select all when editing opens (inline) with a non-empty value. */
  selectOnOpen?: boolean;
  /** Stops every key at the field. */
  stopKeyPropagation?: boolean;
  /** A changed value asks an inline field to enter editing. */
  editRequest?: number;
  onValueChange?: (value: string) => void;
  /** Enter without Shift, value trimmed. */
  onSubmit?: (value: string) => void;
  /** Inline and draft: Enter or blur, value trimmed, at most once per editing session. */
  onCommit?: (value: string) => void;
  /** Escape. */
  onCancel?: () => void;
  className?: string;
}

export const Input = React.forwardRef<Element, InputProps>(function Input(
  {
    size = "md", mode = "inline", multiline = false, value = "", ariaLabel = "", placeholder = "",
    autoFocus = false, selectOnFocus = false, selectOnOpen = true, stopKeyPropagation = false,
    spellCheck = true, disabled = false, editRequest = 0, className,
    onValueChange, onSubmit, onCommit, onCancel, ...props
  },
  ref,
) {
  const node = React.useRef<Element | null>(null);
  const [editing, setEditing] = React.useState(false);
  const pointerDown = React.useRef(false);
  const committed = React.useRef(false);
  const sessionStart = React.useRef("");
  const lastRequest = React.useRef(editRequest);

  const setRef = React.useCallback(
    (el: Element | null) => {
      node.current = el;
      if (typeof ref === "function") ref(el);
      else if (ref) ref.current = el;
    },
    [ref],
  );

  const commit = (next: string) => {
    if (committed.current) return;
    committed.current = true;
    onCommit?.(next);
  };

  React.useEffect(() => {
    if (autoFocus) node.current?.focus();
  }, [autoFocus]);

  React.useEffect(() => {
    if (lastRequest.current === editRequest) return;
    lastRequest.current = editRequest;
    if (mode === "inline" && !disabled) node.current?.focus();
  }, [editRequest, mode, disabled]);

  const handleFocus = () => {
    committed.current = false;
    sessionStart.current = value;
    if (mode === "inline") setEditing(true);
    const select = mode === "inline" ? selectOnOpen : selectOnFocus;
    if (select && value !== "" && !pointerDown.current) node.current?.select();
  };

  const handleMouseDown = (e: React.MouseEvent<Element>) => {
    // A resting inline field opens for editing on press, with the whole text selected.
    if (mode === "inline" && !editing) {
      e.preventDefault();
      node.current?.focus();
      return;
    }
    pointerDown.current = true;
  };

  const handleMouseUp = () => {
    pointerDown.current = false;
  };

  const handleKeyDown = (e: React.KeyboardEvent<Element>) => {
    if (stopKeyPropagation) e.stopPropagation();
    if (e.nativeEvent.isComposing) return;
    if (e.key === "Enter" && (multiline || !e.shiftKey)) {
      e.preventDefault();
      const trimmed = value.trim();
      onSubmit?.(trimmed);
      if (mode !== "bare") commit(trimmed);
      if (mode === "inline") node.current?.blur();
    } else if (e.key === "Escape") {
      if (mode !== "bare") committed.current = true;
      onCancel?.();
      if (mode === "inline") {
        onValueChange?.(sessionStart.current);
        node.current?.blur();
      }
    }
  };

  const handleBlur = () => {
    pointerDown.current = false;
    if (mode !== "bare") commit(value.trim());
    if (mode === "inline") setEditing(false);
  };

  const filled = mode === "draft" || (mode === "inline" && editing);
  const shared = {
    ...props,
    ref: setRef,
    "data-slot": "field",
    value,
    placeholder,
    disabled,
    spellCheck,
    readOnly: mode === "inline" && !editing,
    "aria-label": ariaLabel || undefined,
    className: cn(inputVariants({ size, fill: filled ? "resting" : "none", multiline }), className),
    onChange: (e: React.ChangeEvent<Element>) => onValueChange?.(e.target.value),
    onFocus: handleFocus,
    onBlur: handleBlur,
    onKeyDown: handleKeyDown,
    onMouseDown: handleMouseDown,
    onMouseUp: handleMouseUp,
  };

  return multiline ? (
    <textarea {...shared} rows={1} aria-multiline="true" />
  ) : (
    <input {...shared} type="text" />
  );
});

export { inputVariants };
