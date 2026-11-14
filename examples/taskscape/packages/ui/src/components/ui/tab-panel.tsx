import * as React from "react";
import * as TabsPrimitive from "@radix-ui/react-tabs";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/** The scrolling content pane of the active Settings section (WAI-ARIA tabpanel). */
export const tabPanelVariants = cva(
  [
    "h-full min-h-0 px-8 py-7 outline-none",
    "focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1",
  ],
  {
    variants: {
      tone: {
        "surface-2": "bg-surface-2",
      },
      scroll: {
        true: "overflow-y-auto",
        false: "overflow-hidden",
      },
    },
    defaultVariants: { tone: "surface-2", scroll: true },
  },
);

type Variants = VariantProps<typeof tabPanelVariants>;

export interface TabPanelProps extends Omit<React.ComponentPropsWithoutRef<typeof TabsPrimitive.Content>, "value"> {
  /** Section id; the value of the tab that labels this panel. */
  value?: string;
  /** true: the panel is the vertical scroll region and takes focus when its content overflows. */
  scroll?: boolean;
  tone?: Variants["tone"];
  /** Emitted only when atBottom changes. */
  onScrolled?: (detail: { atBottom: boolean }) => void;
}

export const TabPanel = React.forwardRef<HTMLDivElement, TabPanelProps>(function TabPanel(
  { value = "", scroll = true, tone = "surface-2", onScrolled, onScroll, className, children, ...props },
  forwardedRef,
) {
  const node = React.useRef<HTMLDivElement | null>(null);
  const atBottom = React.useRef(false);
  const [overflowing, setOverflowing] = React.useState(false);

  const setRef = React.useCallback(
    (el: HTMLDivElement | null) => {
      node.current = el;
      if (typeof forwardedRef === "function") forwardedRef(el);
      else if (forwardedRef) forwardedRef.current = el;
    },
    [forwardedRef],
  );

  // Track whether the content overflows, so the region can take keyboard focus only then.
  React.useEffect(() => {
    const el = node.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const measure = () => setOverflowing(el.scrollHeight > el.clientHeight + 1);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    if (el.firstElementChild) observer.observe(el.firstElementChild);
    return () => observer.disconnect();
  }, [scroll, children]);

  const handleScroll = (event: React.UIEvent<HTMLDivElement>) => {
    onScroll?.(event);
    if (!scroll) return;
    const el = event.currentTarget;
    const bottom = el.scrollTop + el.clientHeight >= el.scrollHeight - 1;
    if (bottom !== atBottom.current) {
      atBottom.current = bottom;
      onScrolled?.({ atBottom: bottom });
    }
  };

  return (
    <TabsPrimitive.Content
      {...props}
      ref={setRef}
      value={value}
      data-slot="panel"
      tabIndex={scroll && overflowing ? 0 : -1}
      onScroll={handleScroll}
      className={cn(tabPanelVariants({ tone, scroll }), className)}
    >
      {children}
    </TabsPrimitive.Content>
  );
});
