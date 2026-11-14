import * as React from "react";
import * as CollapsiblePrimitive from "@radix-ui/react-collapsible";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";
import { buttonVariants } from "./button";

/**
 * Expand or collapse control on a parent task row. A Radix Collapsible trigger styled with the
 * icon button classes. Collapsed: always visible. Expanded: the chevron is rotated -90 and the
 * control is shown on row hover or keyboard focus. Not rendered when hidden. Click and double-click
 * stop propagation so the row does not open.
 */
export interface DisclosureButtonProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "hidden"> {
  /** Whether the row's subtasks are shown. */
  expanded?: boolean;
  /** Called with the next expanded value when the control is pressed. */
  onExpandedChange?: (expanded: boolean) => void;
  /** True when the row has no visible children; the control is not rendered. */
  hidden?: boolean;
}

export const DisclosureButton = React.forwardRef<HTMLButtonElement, DisclosureButtonProps>(
  function DisclosureButton(
    { expanded = false, onExpandedChange, hidden = false, className, onClick, onDoubleClick, ...props },
    ref,
  ) {
    if (hidden) return null;

    return (
      <CollapsiblePrimitive.Root open={expanded} onOpenChange={onExpandedChange}>
        <CollapsiblePrimitive.Trigger
          ref={ref}
          data-slot="disclosure"
          aria-label={expanded ? "Collapse" : "Expand"}
          onClick={(event: React.MouseEvent<HTMLButtonElement>) => {
            event.stopPropagation();
            onClick?.(event);
          }}
          onDoubleClick={(event: React.MouseEvent<HTMLButtonElement>) => {
            event.stopPropagation();
            onDoubleClick?.(event);
          }}
          className={cn(
            buttonVariants({ shape: "icon", tone: "default", reveal: expanded ? "row" : "always" }),
            // 18px box; the before pseudo-element pads the hit area to the 24px target minimum.
            "h-[1.2857rem] w-[1.2857rem] shrink-0 before:absolute before:-inset-[0.2143rem] before:content-['']",
            className,
          )}
          {...props}
        >
          <Icon
            name="chevron_right"
            size={16}
            weight={300}
            tone="inherit"
            className={cn(
              "transition-transform duration-(--duration-fast) ease-standard",
              expanded && "-rotate-90",
            )}
          />
        </CollapsiblePrimitive.Trigger>
      </CollapsiblePrimitive.Root>
    );
  },
);
