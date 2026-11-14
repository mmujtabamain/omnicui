import * as React from "react";
import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import { cn } from "../../lib/utils";

/** One per app (and per harness): sets the open delay for every tooltip under it. */
export const TooltipProvider = ({ delayDuration = 500, ...props }: React.ComponentProps<typeof TooltipPrimitive.Provider>) => (
  <TooltipPrimitive.Provider delayDuration={delayDuration} {...props} />
);

export interface TooltipProps {
  /** The tooltip text; no tooltip when empty. */
  content: React.ReactNode;
  side?: "top" | "right" | "bottom" | "left";
  children: React.ReactElement;
}

/** A text tooltip on its trigger: opens on hover and keyboard focus, closes on Escape. */
export function Tooltip({ content, side = "top", children }: TooltipProps) {
  if (content === "" || content == null) return children;
  return (
    <TooltipPrimitive.Root>
      <TooltipPrimitive.Trigger asChild>{children}</TooltipPrimitive.Trigger>
      <TooltipPrimitive.Portal>
        <TooltipPrimitive.Content
          data-slot="tooltip"
          side={side}
          sideOffset={6}
          collisionPadding={6}
          className={cn(
            "z-(--z-index-tooltip) max-w-72 rounded-(--radius-field) bg-content-1 px-2 py-1 text-[0.857rem] leading-snug text-surface-2 shadow-lift",
            "data-[state=delayed-open]:animate-rise data-[state=instant-open]:animate-rise",
          )}
        >
          {content}
        </TooltipPrimitive.Content>
      </TooltipPrimitive.Portal>
    </TooltipPrimitive.Root>
  );
}
