import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";
import { Input } from "./input";

/**
 * Single-line title field inserted under a parent task row to add a subtask. Row height 36px;
 * left inset depth * 24 + 6 px; leading icon subdirectory_arrow_right at 15px, weight 300.
 * Enter with a title submits it and clears the field; Enter on an empty or blank field, Escape,
 * or focus leaving the composer dismisses it. The caller owns whether the composer is shown.
 */
const inlineComposerVariants = cva("flex w-full min-w-0 items-center gap-1.5 pr-2", {
  variants: {
    kind: {
      child: "h-[2.571rem]",
    },
  },
  defaultVariants: { kind: "child" },
});

type Variants = VariantProps<typeof inlineComposerVariants>;

export interface InlineComposerProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "onSubmit"> {
  /** The only recorded value is child. */
  kind?: Variants["kind"];
  /** Indent level. */
  depth?: number;
  /** Focus the field on mount. */
  autoFocus?: boolean;
  /** Enter with a title; the title is trimmed. */
  onSubmit?: (title: string) => void;
  /** Empty Enter, Escape or blur. */
  onDismiss?: () => void;
}

export const InlineComposer = React.forwardRef<HTMLDivElement, InlineComposerProps>(function InlineComposer(
  { kind = "child", depth = 0, autoFocus = true, onSubmit, onDismiss, className, style, onBlur, onFocus, ...props },
  ref,
) {
  const [title, setTitle] = React.useState("");
  // A dismissal ends a session: blur and Escape report once per session. Typing or focusing the
  // field starts a new session, so a caller that keeps the composer mounted gets later dismissals.
  const dismissed = React.useRef(false);

  const dismiss = () => {
    if (dismissed.current) return;
    dismissed.current = true;
    onDismiss?.();
  };

  const submit = (text: string) => {
    if (text === "") {
      dismiss();
      return;
    }
    setTitle("");
    onSubmit?.(text);
  };

  return (
    <div
      ref={ref}
      className={cn(inlineComposerVariants({ kind }), className)}
      style={{ paddingLeft: `${(depth * 24 + 6) / 14}rem`, ...style }}
      onFocus={(e) => {
        onFocus?.(e);
        dismissed.current = false;
      }}
      onBlur={(e) => {
        onBlur?.(e);
        const next = e.relatedTarget;
        if (!(next instanceof Node && e.currentTarget.contains(next))) dismiss();
      }}
      {...props}
    >
      <span className="flex shrink-0 items-center text-content-3">
        <Icon data-slot="icon" name="subdirectory_arrow_right" size={15} weight={300} tone="inherit" />
      </span>
      <Input
        mode="bare"
        size="md"
        className="flex-1"
        ariaLabel="Subtask title"
        placeholder="Subtask title"
        value={title}
        autoFocus={autoFocus}
        stopKeyPropagation
        onValueChange={(value) => {
          dismissed.current = false;
          setTitle(value);
        }}
        onSubmit={submit}
        onCancel={dismiss}
      />
    </div>
  );
});

export { inlineComposerVariants };
