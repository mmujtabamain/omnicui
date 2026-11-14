import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";
import { Button } from "./button";

/**
 * scope: "pane" fills the task pane on the surface-2 fill, with no icon; "list" sits inline in a scroll
 * area with the measured list spacing. surface: "plain" has no fill; "card" is the lightbox fallback panel.
 */
export const emptyStateVariants = cva("flex min-w-0 flex-col items-center gap-1 text-center", {
  variants: {
    scope: {
      pane: "h-full w-full justify-center bg-surface-2",
      list: "pt-24 pb-10",
    },
    surface: {
      plain: "",
      card: "rounded-(--radius-panel) bg-surface-2 px-10 py-12",
    },
  },
  defaultVariants: { scope: "list", surface: "plain" },
});

export interface EmptyStateProps
  extends Omit<React.HTMLAttributes<HTMLDivElement>, "title">,
    VariantProps<typeof emptyStateVariants> {
  /** 15px medium secondary text; "Loading…" while loading. */
  title?: string;
  /** Optional muted 12.5px line; empty means none. */
  subtitle?: string;
  /** Material Symbols glyph name: "", "landscape", "delete", "search" or "search_off". */
  icon?: string;
  /** Icon size in px (30 for the list icon, 28 for the aside). */
  iconSize?: number;
  /** The title is the loading text, announced through a status region with aria-busy. */
  loading?: boolean;
  /** Label of the optional action button; empty means no button. */
  actionLabel?: string;
  /** Called when the action button is pressed. */
  onAction?: () => void;
}

export const EmptyState = React.forwardRef<HTMLDivElement, EmptyStateProps>(function EmptyState(
  {
    scope = "list",
    surface = "plain",
    title = "",
    subtitle = "",
    icon = "",
    iconSize = 30,
    loading = false,
    actionLabel = "",
    onAction,
    className,
    ...props
  },
  ref,
) {
  const showIcon = icon !== "" && scope !== "pane" && !loading;
  const showSubtitle = subtitle !== "" && !loading;

  const titleNode = (
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
      {showIcon && <Icon data-slot="icon" name={icon} size={iconSize} className="mb-2" />}
      {loading ? (
        <div role="status" aria-busy="true" data-slot="loadingStatus" className="flex min-w-0 flex-col items-center">
          {titleNode}
        </div>
      ) : (
        titleNode
      )}
      {showSubtitle && (
        <div data-slot="subtitle" className="text-[0.893rem] text-content-3">
          {subtitle}
        </div>
      )}
      {actionLabel !== "" && (
        <Button
          data-slot="action"
          variant="outline"
          size="md"
          width="content"
          label={actionLabel}
          onClick={onAction}
          className="mt-2"
        />
      )}
    </div>
  );
});
