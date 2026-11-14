import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";
import { InputGroup } from "./input-group";

/**
 * The input row at the top of the command palette: InputGroup's bar shape (a 1px bottom rule)
 * at the measured 44px, which is 3.143rem at 1rem = 14px. The palette has no clear button, so the
 * trailing slot InputGroup shows once a query is typed is hidden. Focus is shown on the 1px rule
 * (its colour turns to the focus token) instead of InputGroup's 2px ring around the whole row.
 */
export const commandInputVariants = cva(
  "h-[3.143rem] [&_[data-slot=trailing]]:hidden focus-within:ring-0 focus-within:border-b-focus-1",
  {
    variants: {
      leading: {
        icon: "gap-2",
      },
    },
    defaultVariants: { leading: "icon" },
  },
);

type Variants = VariantProps<typeof commandInputVariants>;

export interface CommandInputProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "defaultValue" | "onSubmit"> {
  /** The query. Controlled: the host passes it back on every queryChanged. */
  value?: string;
  /** Shown while the query is empty. */
  placeholder?: string;
  /** Leading glyph. icon is the bolt glyph, the only value in the spec. */
  leading?: Variants["leading"];
  /** Takes focus when the palette opens. */
  autoFocus?: boolean;
  /** Accessible name of the input. Required; the palette root supplies it. */
  ariaLabel?: string;
  /** True while the listbox is shown. Supplied by the palette root. */
  ariaExpanded?: boolean;
  /** Id of the listbox. Supplied by the palette root. */
  ariaControls?: string;
  /** Id of the active option; empty means no active row. Supplied by the host. */
  activeDescendantId?: string;
  /** Every user edit of the query. */
  onValueChange?: (value: string) => void;
  /** ArrowDown or ArrowUp, on every press. The host clamps the active row. */
  onMoveActive?: (direction: "up" | "down") => void;
  /** Enter, default prevented. Carries the query; the host runs the active row. */
  onSubmit?: (value: string) => void;
}

function setOrRemove(el: Element, name: string, value: string) {
  if (value) el.setAttribute(name, value);
  else el.removeAttribute(name);
}

export const CommandInput = React.forwardRef<HTMLDivElement, CommandInputProps>(function CommandInput(
  {
    value = "",
    placeholder = "",
    leading = "icon",
    autoFocus = true,
    ariaLabel = "",
    ariaExpanded = false,
    ariaControls = "",
    activeDescendantId = "",
    onValueChange,
    onMoveActive,
    onSubmit,
    onKeyDown,
    className,
    ...props
  },
  forwardedRef,
) {
  const rootRef = React.useRef<HTMLDivElement | null>(null);
  const setRootRef = React.useCallback(
    (node: HTMLDivElement | null) => {
      rootRef.current = node;
      if (typeof forwardedRef === "function") forwardedRef(node);
      else if (forwardedRef) forwardedRef.current = node;
    },
    [forwardedRef],
  );

  const field = () => rootRef.current?.querySelector<HTMLInputElement>('[data-slot="input"]') ?? null;

  // The combobox state belongs on the inner input, which InputGroup does not expose: set it there.
  React.useLayoutEffect(() => {
    const el = field();
    if (!el) return;
    el.setAttribute("aria-autocomplete", "list");
    el.setAttribute("aria-expanded", String(ariaExpanded));
    setOrRemove(el, "aria-controls", ariaControls);
    setOrRemove(el, "aria-activedescendant", activeDescendantId);
  }, [ariaExpanded, ariaControls, activeDescendantId]);

  React.useEffect(() => {
    if (autoFocus) field()?.focus();
  }, [autoFocus]);

  // Key presses bubble from the inner input to the row, so the keys are read here.
  function handleKeyDown(e: React.KeyboardEvent<HTMLDivElement>) {
    onKeyDown?.(e);
    if (e.nativeEvent.isComposing) return;
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        onMoveActive?.("down");
        break;
      case "ArrowUp":
        e.preventDefault();
        onMoveActive?.("up");
        break;
      case "Enter":
        e.preventDefault();
        onSubmit?.(value);
        break;
    }
  }

  return (
    <InputGroup
      ref={setRootRef}
      data-slot="row"
      shape="bar"
      size="md"
      leading={leading === "icon" ? <Icon name="bolt" size={16} tone="inherit" /> : undefined}
      value={value}
      placeholder={placeholder}
      inputLabel={ariaLabel}
      onValueChange={onValueChange}
      onKeyDown={handleKeyDown}
      className={cn(commandInputVariants({ leading }), className)}
      {...props}
    />
  );
});
