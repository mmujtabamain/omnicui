import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { Icon } from "./icon";

/**
 * The search well: a pill in the search tone (wash fill, no border) with a leading magnifier,
 * a native input type=search and an optional clear button. Sizes follow InputGroup's search tone
 * (md 2.5rem high, sm 1.75rem high). Not built on InputGroup or Input: both render their own
 * type=text input with an explicit role, and this field must stay a native type=search with no role.
 */
export const searchFieldVariants = cva(
  [
    "group/search-field relative flex min-w-0 items-center rounded-full bg-wash-2 outline-none",
    "transition-[background-color,opacity] duration-(--duration-fast) ease-standard",
    "focus-within:ring-2 focus-within:ring-focus-1",
    "data-[disabled]:pointer-events-none data-[disabled]:opacity-45",
  ],
  {
    variants: {
      size: {
        md: "h-[2.5rem] gap-[0.5rem] pl-[0.5rem] pr-[0.5rem]",
        sm: "h-[1.75rem] gap-[0.375rem] px-[0.375rem]",
      },
    },
    defaultVariants: { size: "md" },
  },
);

const FIELD = [
  "min-w-0 flex-1 self-center appearance-none bg-transparent p-0 text-content-1 outline-none",
  "placeholder:text-content-3 selection:bg-selection-1 disabled:cursor-not-allowed",
  "[&::-webkit-search-cancel-button]:appearance-none [&::-webkit-search-decoration]:appearance-none",
].join(" ");

const FIELD_SIZE = {
  md: "h-[1.5rem] text-[1rem]",
  sm: "h-[1.286rem] text-[0.857rem]",
} as const;

const ICON_SIZE = { md: 18, sm: 14 } as const;

const ICON_SLOT = "inline-flex shrink-0 items-center text-content-3";

export interface SearchFieldProps
  extends Omit<React.HTMLAttributes<HTMLDivElement>, "defaultValue" | "onChange"> {
  /** Controlled query. */
  value?: string;
  placeholder?: string;
  /** md: list panes, 14px text. sm: settings sidebar, 12px text. Width follows the pane. */
  size?: "md" | "sm";
  /** filter: typing filters the open list. match: Enter and Shift+Enter step through matches. */
  navigation?: "filter" | "match";
  /** Shows a trailing clear button while the value is non-empty. */
  clearable?: boolean;
  autoFocus?: boolean;
  disabled?: boolean;
  /** Accessible name, required in the React API. */
  ariaLabel?: string;
  /** Every change of the query, including clears. */
  onValueChange?: (value: string) => void;
  /** match navigation only: Enter gives next, Shift+Enter gives previous. */
  onStepMatch?: (direction: "next" | "previous") => void;
  /** Escape or the clear button emptied the query. */
  onClear?: () => void;
}

export const SearchField = React.forwardRef<HTMLInputElement, SearchFieldProps>(function SearchField(
  {
    value = "", placeholder = "Search", size = "md", navigation = "filter", clearable = false,
    autoFocus = false, disabled = false, ariaLabel = "", onValueChange, onStepMatch, onClear,
    className, onMouseDown, ...props
  },
  ref,
) {
  const inputRef = React.useRef<HTMLInputElement | null>(null);
  const setInputRef = React.useCallback(
    (el: HTMLInputElement | null) => {
      inputRef.current = el;
      if (typeof ref === "function") ref(el);
      else if (ref) ref.current = el;
    },
    [ref],
  );

  const sizeName = size === "sm" ? "sm" : "md";
  const showClear = clearable && value !== "";

  const empty = () => {
    onValueChange?.("");
    onClear?.();
  };

  // A press on the well (outside the clear button) focuses the field.
  function handleWellMouseDown(e: React.MouseEvent<HTMLDivElement>) {
    onMouseDown?.(e);
    if (disabled) return;
    if ((e.target as Element).closest("button, input")) return;
    e.preventDefault();
    inputRef.current?.focus();
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.nativeEvent.isComposing) return;
    if (navigation === "match") {
      e.stopPropagation();
      if (e.key === "Enter") {
        e.preventDefault();
        onStepMatch?.(e.shiftKey ? "previous" : "next");
      } else if (e.key === "Escape") {
        e.preventDefault();
        if (value !== "") empty();
        e.currentTarget.blur();
      }
      return;
    }
    // filter: Escape clears a non-empty query of a clearable field; an empty one bubbles.
    if (e.key === "Escape" && clearable && value !== "") {
      e.preventDefault();
      e.stopPropagation();
      empty();
    }
  }

  return (
    <div
      data-slot="well"
      data-disabled={disabled || undefined}
      onMouseDown={handleWellMouseDown}
      className={cn(searchFieldVariants({ size: sizeName }), className)}
      {...props}
    >
      <span data-slot="icon" data-part="ui.searchfield.icon" aria-hidden="true" className={ICON_SLOT}>
        <Icon name="search" size={ICON_SIZE[sizeName]} tone="inherit" />
      </span>
      <input
        ref={setInputRef}
        data-slot="field"
        type="search"
        aria-label={ariaLabel || undefined}
        autoComplete="off"
        spellCheck={false}
        autoFocus={autoFocus}
        disabled={disabled}
        placeholder={placeholder}
        value={value}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => onValueChange?.(e.target.value)}
        onKeyDown={handleKeyDown}
        className={cn(FIELD, FIELD_SIZE[sizeName])}
      />
      {showClear && (
        <Button
          data-slot="clearButton"
          ariaLabel="Clear search"
          icon="close"
          shape="icon"
          size={sizeName}
          tone="muted"
          retainFocus
          disabled={disabled}
          onClick={() => {
            empty();
            inputRef.current?.focus();
          }}
        />
      )}
    </div>
  );
});
