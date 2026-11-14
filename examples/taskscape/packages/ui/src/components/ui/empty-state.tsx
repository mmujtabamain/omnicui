import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { Icon } from "./icon";

/**
 * Pane fills the task pane on the surface-2 fill, with no icon. List is inline in a scroll area,
 * with the measured list spacing. Surface plain has no fill; card is the lightbox fallback card.
 */
export const emptyStateVariants = cva("flex min-w-0 flex-col items-center text-center", {
  variants: {
    scope: {
      pane: "h-full w-full justify-center bg-surface-2 px-4",
      list: "pt-24 pb-10 px-4",
    },
    surface: {
      plain: "",
      card: "rounded-(--radius-panel) bg-surface-2 px-10 py-12",
    },
  },
  defaultVariants: { scope: "list", surface: "plain" },
});

export type EmptyStateProps = Omit<React.HTMLAttributes<HTMLDivElement>, "title"> &
  VariantProps<typeof emptyStateVariants> & {
    /** 15px medium, secondary text. */
    title?: string;
    /** Optional muted 12.5px line. Empty means no subtitle. */
    subtitle?: string;
    /** Material Symbols glyph name: "landscape", "delete", "search" or "search_off". */
    icon?: string;
    /** Icon size in px. */
    iconSize?: number;
    /** The title is the loading text, announced through a status region. */
    loading?: boolean;
    /** Label of the optional action button. Empty means no button. */
    actionLabel?: string;
    /** Press handler for the action button. */
    onAction?: React.MouseEventHandler<HTMLButtonElement>;
    className?: string;
  };

export const EmptyState = React.forwardRef<HTMLDivElement, EmptyStateProps>(function EmptyState(
  {
    scope, surface, title = "", subtitle = "", icon = "", iconSize = 30, loading = false,
    actionLabel = "", onAction, className, ...props
  },
  ref,
) {
  const showIcon = icon !== "" && scope !== "pane" && !loading;
  const showSubtitle = subtitle !== "" && !loading;

  const titleText = (
    <div data-slot="title" className="text-[1.071rem] font-medium text-content-2">
      {title}
    </div>
  );

  return (
    <div
      ref={ref}
      data-slot="root"
      className={cn(emptyStateVariants({ scope, surface }), className)}
      {...props}
    >
      {showIcon && (
        <div className="mb-2 flex justify-center text-content-3">
          <Icon data-slot="icon" name={icon} size={iconSize} tone="inherit" aria-hidden="true" />
        </div>
      )}
      {loading ? (
        <div data-slot="loadingStatus" role="status" aria-busy="true" className="flex flex-col items-center">
          {titleText}
        </div>
      ) : (
        titleText
      )}
      {showSubtitle && (
        <div data-slot="subtitle" className="mt-1 text-[0.893rem] text-content-3">
          {subtitle}
        </div>
      )}
      {actionLabel !== "" && (
        <Button
          data-slot="action"
          variant="outline"
          size="md"
          label={actionLabel}
          onClick={onAction}
          className="mt-4"
        />
      )}
    </div>
  );
});
