import * as React from "react";
import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";

/** The Settings wording as the desktop apps word it. Web shows its own copy for it (contract decision 4). */
const SETTINGS_WORDING = "Reduced motion is on (Settings > General)";
const WEB_SETTINGS_COPY = "Reduced motion is on (set in your account settings)";

export interface StatusIndicatorProps extends React.HTMLAttributes<HTMLSpanElement> {
  /** Glyph name, rendered by Icon at xl (20px inside the 32px cell). */
  icon?: string;
  /** Accessible name of the status (the cell's aria-label). */
  label?: string;
  /** Explanation shown on hover and on focus: the Low Power or the Settings wording. */
  tooltip?: string;
  /** Shows the cell only while the status is active; false hides it (nothing is drawn). */
  active?: boolean;
}

/**
 * A non-interactive 32px status cell with a 20px glyph. It is a tab stop: focus opens the tooltip,
 * a tap opens it too, and Escape closes it. The tooltip never takes focus and has no motion.
 * While inactive the cell and the tooltip stay in the tree but are hidden (display none).
 */
export const StatusIndicator = React.forwardRef<HTMLSpanElement, StatusIndicatorProps>(function StatusIndicator(
  { icon = "motion_photos_off", label = "Reduced motion", tooltip = SETTINGS_WORDING, active = true, className, ...props },
  ref,
) {
  const [open, setOpen] = React.useState(false);
  // Set on a touch tap: the click that follows the tap must not close the tooltip it just opened.
  const tapRef = React.useRef(false);

  const text = tooltip === SETTINGS_WORDING ? WEB_SETTINGS_COPY : tooltip;

  return (
    <TooltipPrimitive.Root
      open={open && active}
      onOpenChange={(next) => {
        if (!next && tapRef.current) {
          tapRef.current = false;
          return;
        }
        setOpen(next);
      }}
    >
      <TooltipPrimitive.Trigger asChild>
        <span
          {...props}
          ref={ref}
          data-slot="cell"
          aria-label={label}
          tabIndex={0}
          onPointerUp={(e) => {
            if (e.pointerType === "touch") {
              tapRef.current = true;
              setOpen(true);
            }
          }}
          className={cn(
            active ? "inline-flex" : "hidden",
            "h-[2.2857rem] w-[2.2857rem] shrink-0 items-center justify-center rounded-(--radius-control) outline-none focus-visible:ring-2 focus-visible:ring-focus-1",
            className,
          )}
        >
          <Icon data-slot="glyph" name={icon} size="xl" tone="secondary" />
        </span>
      </TooltipPrimitive.Trigger>
      <TooltipPrimitive.Portal forceMount>
        <TooltipPrimitive.Content
          data-slot="tooltip"
          role="status"
          side="top"
          sideOffset={6}
          collisionPadding={6}
          className="z-(--z-index-tooltip) max-w-72 rounded-(--radius-field) bg-content-1 px-2 py-1 text-[0.857rem] leading-snug text-surface-2 shadow-lift data-[state=closed]:hidden"
        >
          {text}
        </TooltipPrimitive.Content>
      </TooltipPrimitive.Portal>
    </TooltipPrimitive.Root>
  );
});
