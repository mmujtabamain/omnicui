import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { Text } from "./text";

/** Eyebrow heading for a content block. Sizes are rem at 1rem = 14px; colours are tokens only. */
export const sectionHeaderVariants = cva("flex min-w-0 items-center gap-2", {
  variants: {
    inset: {
      block: "",
      listGroup: "mt-2 mb-1 px-3",
    },
  },
  defaultVariants: { inset: "block" },
});

/** The trailing hairline. hairline runs a fixed run and shrinks in a narrow header (so the restore control stays in view); trailing runs from the label to the pane edge. */
export const sectionHeaderRuleVariants = cva("block h-px bg-edge-2", {
  variants: {
    rule: {
      none: "",
      hairline: "w-[17.9rem] min-w-0 max-w-full",
      trailing: "min-w-4 flex-1",
    },
  },
  defaultVariants: { rule: "none" },
});

export type SectionHeaderRule = "none" | "hairline" | "trailing";
export type SectionHeaderRestore = "none" | "reset" | "discard";

export interface SectionHeaderProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "children"> {
  /** Heading text. */
  label?: React.ReactNode;
  /** Trailing slot: count text (rendered muted) or add buttons. */
  trailing?: React.ReactNode;
  /** Non-default group: shows the changed dot and the restore control. */
  active?: boolean;
  /** reset (filter groups) or discard (settings groups); none shows no restore control. */
  restore?: SectionHeaderRestore;
  /** Accessible name and tooltip of the restore control. */
  restoreLabel?: string;
  /** Heading level for block headings. div for listGroup. */
  as?: "h2" | "h3" | "div";
  /** block (caps, tracking) or listGroup (inset, spacing, no rule). */
  inset?: "block" | "listGroup";
  /** Trailing hairline: none, hairline after the label, or trailing to the pane edge. */
  rule?: SectionHeaderRule;
  /** Runs the restore for reset or discard. */
  onRestore?: () => void;
}

export const SectionHeader = React.forwardRef<HTMLDivElement, SectionHeaderProps>(function SectionHeader(
  {
    label = "",
    trailing = "",
    active = false,
    restore = "none",
    restoreLabel = "",
    as = "h3",
    inset = "block",
    rule = "none",
    onRestore,
    className,
    ...props
  },
  ref,
) {
  const isList = inset === "listGroup";
  const isHeading = !isList && (as === "h2" || as === "h3");
  const hasTrailing = trailing !== "" && trailing !== null && trailing !== undefined && trailing !== false;
  const isText = typeof trailing === "string" || typeof trailing === "number";
  const showRule = !isList && rule !== "none";
  const showRestore = active && restore !== "none";

  return (
    <div ref={ref} className={cn(sectionHeaderVariants({ inset }), className)} {...props}>
      <Text
        data-slot="title"
        as="div"
        role={isHeading ? "heading" : undefined}
        aria-level={isHeading ? (as === "h2" ? 2 : 3) : undefined}
        size="xs"
        weight="semibold"
        tone="muted"
        overflow="truncate"
        // Caps by font-variant, not text-transform: the label text stays as written ("Notes").
        className="tracking-[0.08em] [font-variant-caps:all-small-caps]"
      >
        {label}
      </Text>

      {active && (
        <span data-slot="changedDot" aria-hidden="true" className="block size-[0.429rem] shrink-0 rounded-full bg-content-1" />
      )}

      {showRule && <span data-slot="rule" aria-hidden="true" className={sectionHeaderRuleVariants({ rule })} />}

      {(hasTrailing || showRestore) && (
        <div className="ml-auto flex shrink-0 items-center gap-1">
          {hasTrailing && (
            <div data-slot="trailing" className="flex min-w-0 items-center gap-1">
              {isText ? (
                <Text size="xs" tone="muted" numeric="tabular">
                  {trailing}
                </Text>
              ) : (
                trailing
              )}
            </div>
          )}
          {showRestore && (
            <Button
              data-slot="restore"
              variant="ghost"
              size="sm"
              tone="muted"
              shape="icon"
              icon={restore === "reset" ? "restart_alt" : "undo"}
              ariaLabel={restoreLabel}
              description={restoreLabel}
              onClick={onRestore}
            />
          )}
        </div>
      )}
    </div>
  );
});
