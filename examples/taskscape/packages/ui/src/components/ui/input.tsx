import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * Single-line text entry with no chrome of its own. The host supplies the surface (InputGroup,
 * SearchField, the tab bar). Base: shadcn Input; `multiline` renders a textarea that grows from one
 * line (21px) to two (42px) and then scrolls.
 */
const inputVariants = cva(
  [
    "block w-full min-w-0 bg-transparent text-content-1 placeholder:text-content-3 selection:bg-selection-1",
    "rounded-(--radius-field) outline-none focus:ring-1 focus:ring-focus-1",
    "transition-[background-color,box-shadow] duration-(--duration-fast) ease-standard",
    "disabled:cursor-default disabled:opacity-45",
  ],
  {
    variants: {
      size: {
        inline: "h-[1.5rem] max-w-[10rem] px-space-4 py-0 text-[1rem] font-medium leading-[1.5rem]",
        md: "h-[1.5rem] px-0 text-[1rem] leading-[1.5rem]",
        sm: "h-[1.286rem] px-0 text-[0.857rem] leading-[1.286rem]",
        display: "h-[1.5rem] px-0 text-[1.286rem] font-semibold leading-[1.5rem]",
      },
      mode: {
        inline: "cursor-text bg-transparent px-space-4",
        draft: "bg-surface-0 px-space-4",
        bare: "bg-transparent px-0",
      },
      editing: {
        true: "bg-surface-0",
        false: "",
      },
      multiline: {
        true: "h-auto min-h-[1.5rem] max-h-[3rem] field-sizing-content resize-none overflow-y-auto py-0 break-words whitespace-pre-wrap",
        false: "",
      },
    },
    defaultVariants: { size: "md", mode: "inline", editing: false, multiline: false },
  },
);

type InputVariants = VariantProps<typeof inputVariants>;
type FieldElement = HTMLInputElement | HTMLTextAreaElement;

export interface InputProps
  extends Omit<
    React.HTMLAttributes<HTMLElement>,
    | "defaultValue" | "onChange" | "onFocus" | "onBlur" | "onKeyDown" | "onMouseDown" | "onMouseUp"
    | "onSubmit" | "onCancel" | "onCommit" | "onValueChange"
  > {
  /** Controlled text. */
  value?: string;
  /** What the field holds: password masks it; email and url bring the matching keyboard and autofill. */
  type?: "text" | "password" | "email" | "url" | "search";
  /** Autofill hint, e.g. "email", "current-password", "new-password". */
  autoComplete?: string;
  name?: string;
  autoFocus?: boolean;
  disabled?: boolean;
  placeholder?: string;
  /** Accessible name. */
  ariaLabel?: string;
  size?: InputVariants["size"];
  mode?: InputVariants["mode"];
  /** A textarea: one line grows to two by soft wrap, then scrolls. Enter never inserts a line break. */
  multiline?: boolean;
  /** Keyboard focus with a non-empty value selects all. */
  selectOnFocus?: boolean;
  /** Opening for editing with a non-empty value selects all. */
  selectOnOpen?: boolean;
  /** Every key stops at the field. */
  stopKeyPropagation?: boolean;
  spellCheck?: boolean;
  /** A changed value asks an inline field to enter editing. */
  editRequest?: number;
  onValueChange?: (value: string) => void;
  /** Enter without Shift, trimmed. */
  onSubmit?: (value: string) => void;
  /** Inline and draft: Enter or blur, trimmed; at most once per editing session. */
  onCommit?: (value: string) => void;
  /** Escape. */
  onCancel?: () => void;
}

export const Input = React.forwardRef<FieldElement, InputProps>(function Input(
  {
    value = "", placeholder = "", ariaLabel = "", size = "md", mode = "inline", multiline = false,
    autoFocus = false, selectOnFocus = false, selectOnOpen = true, stopKeyPropagation = false,
    spellCheck = true, disabled = false, editRequest = 0, type = "text", autoComplete, className,
    onValueChange, onSubmit, onCommit, onCancel, ...props
  },
  ref,
) {
  const [editing, setEditing] = React.useState(false);
  const fieldRef = React.useRef<FieldElement | null>(null);
  const pointerDown = React.useRef(false);
  const settled = React.useRef(false);
  const seenRequest = React.useRef(editRequest);
  const atRest = mode === "inline" && !editing;

  // Starting an inline edit (a click or a changed editRequest) opens a new editing session.
  const startEditing = () => {
    settled.current = false;
    setEditing(true);
  };

  const setRef = React.useCallback(
    (el: FieldElement | null) => {
      fieldRef.current = el;
      if (typeof ref === "function") ref(el);
      else if (ref) ref.current = el;
    },
    [ref],
  );

  // A changed editRequest starts inline editing. A value present at mount does not.
  React.useEffect(() => {
    if (seenRequest.current === editRequest) return;
    seenRequest.current = editRequest;
    if (mode === "inline" && !disabled) startEditing();
  }, [editRequest, mode, disabled]);

  // Entering edit focuses the field and, with a value, selects all of it.
  React.useEffect(() => {
    if (!editing) return;
    const el = fieldRef.current;
    if (!el) return;
    el.focus();
    if (selectOnOpen && el.value !== "") el.select();
  }, [editing, selectOnOpen]);

  const commitOnce = (text: string) => {
    if (mode === "bare" || settled.current) return;
    settled.current = true;
    onCommit?.(text);
  };

  const handleMouseDown = (e: React.MouseEvent<FieldElement>) => {
    pointerDown.current = true;
    if (mode === "inline" && !editing && !disabled) {
      // Stop the browser placing a caret: the effect above focuses and selects the whole text.
      e.preventDefault();
      startEditing();
    }
  };

  const handleMouseUp = () => {
    pointerDown.current = false;
  };

  const handleFocus = (e: React.FocusEvent<FieldElement>) => {
    const byPointer = pointerDown.current;
    pointerDown.current = false;
    // A readonly inline field is focusable but does not start editing on focus alone.
    if (mode !== "inline") settled.current = false;
    if (selectOnFocus && !byPointer && e.currentTarget.value !== "") e.currentTarget.select();
  };

  const handleBlur = (e: React.FocusEvent<FieldElement>) => {
    if (mode === "inline") {
      if (!editing) return;
      setEditing(false);
    }
    commitOnce(e.currentTarget.value.trim());
  };

  const handleKeyDown = (e: React.KeyboardEvent<FieldElement>) => {
    if (stopKeyPropagation) e.stopPropagation();
    // A readonly inline field at rest has no session: keys do not submit, commit or cancel (blur does not either).
    if (e.nativeEvent.isComposing || atRest) return;
    if (e.key === "Enter" && (multiline || !e.shiftKey)) {
      // Enter never inserts a line break; Shift+Enter on a single line does nothing.
      e.preventDefault();
      const text = e.currentTarget.value.trim();
      onSubmit?.(text);
      commitOnce(text);
    } else if (e.key === "Escape") {
      settled.current = true;
      onCancel?.();
      if (mode === "inline") e.currentTarget.blur();
    }
  };

  const shared = {
    "data-slot": "field",
    "aria-label": ariaLabel || undefined,
    value,
    placeholder,
    spellCheck,
    disabled,
    autoFocus,
    readOnly: atRest,
    className: cn(inputVariants({ size, mode, editing, multiline }), className),
    onChange: (e: React.ChangeEvent<FieldElement>) => onValueChange?.(e.currentTarget.value),
    onKeyDown: handleKeyDown,
    onFocus: handleFocus,
    onBlur: handleBlur,
    onMouseDown: handleMouseDown,
    onMouseUp: handleMouseUp,
  };

  if (multiline)
    return <textarea {...props} {...shared} ref={setRef} rows={1} aria-multiline={true} />;
  return <input {...props} {...shared} ref={setRef} type={type} autoComplete={autoComplete} />;
});

export { inputVariants };
