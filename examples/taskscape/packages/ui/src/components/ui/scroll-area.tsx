import * as React from "react";
import * as ScrollAreaPrimitive from "@radix-ui/react-scroll-area";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";

/** Tone is the fill: base has none; raised sits on surface-2 with content-2 text. */
const scrollAreaVariants = cva("relative size-full min-h-0 min-w-0 overflow-hidden", {
  variants: {
    tone: {
      base: "",
      raised: "bg-surface-2 text-content-2",
    },
  },
  defaultVariants: { tone: "base" },
});

/**
 * The scroll viewport: the region part. The native bar is always hidden; Radix draws the thumb
 * when the bar is shown, and the hidden bar keeps plain overflow scrolling.
 */
const scrollAreaViewportVariants = cva(
  [
    "size-full min-h-0 min-w-0 outline-none",
    "[scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
    "focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1",
    "data-[drop-over=true]:bg-selection-1",
  ],
  {
    variants: {
      overscroll: { contain: "overscroll-contain" },
      scrollbar: { native: "", hidden: "" },
      orientation: { vertical: "", horizontal: "" },
    },
    compoundVariants: [
      { scrollbar: "hidden", orientation: "vertical", className: "overflow-y-auto overflow-x-hidden" },
      { scrollbar: "hidden", orientation: "horizontal", className: "overflow-x-auto overflow-y-hidden" },
    ],
    defaultVariants: { overscroll: "contain", scrollbar: "native", orientation: "vertical" },
  },
);

const SETTLE_MS = 150;
const EDGE_PX = 35;
const EDGE_STEP_PX = 12;

export interface ScrollAreaProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "dir"> {
  /** Scroll axis. */
  orientation?: "vertical" | "horizontal";
  /** Scrollbar visibility; hidden still scrolls with wheel and trackpad. */
  scrollbar?: "native" | "hidden";
  /** Fill token. */
  tone?: "base" | "raised";
  /** Accessible name; makes the region labelled. */
  label?: string;
  /** overscroll-contain, the default. */
  overscroll?: "contain";
  /** Task list: a drop on blank space moves the task to the list root. */
  dropTarget?: boolean;
  onScrollingChanged?: (scrolling: boolean) => void;
  onBottomChanged?: (atBottom: boolean) => void;
  onDropOverChanged?: (over: boolean) => void;
  onRootDropped?: (taskId: string) => void;
  /** A press on blank space (not a data-task-id row). */
  onPressBlank?: () => void;
}

const isRow = (target: EventTarget | null) =>
  target instanceof Element && target.closest("[data-task-id]") !== null;

const rowId = (target: EventTarget | null) =>
  target instanceof Element ? (target.closest("[data-task-id]")?.getAttribute("data-task-id") ?? null) : null;

export const ScrollArea = React.forwardRef<HTMLDivElement, ScrollAreaProps>(function ScrollArea(
  {
    orientation = "vertical",
    scrollbar = "native",
    tone = "base",
    label = "",
    overscroll = "contain",
    dropTarget = false,
    onScrollingChanged,
    onBottomChanged,
    onDropOverChanged,
    onRootDropped,
    onPressBlank,
    className,
    children,
    ...props
  },
  ref,
) {
  const horizontal = orientation === "horizontal";
  const viewportRef = React.useRef<HTMLDivElement>(null);
  const scrollingRef = React.useRef(false);
  const atBottomRef = React.useRef(false);
  const settleRef = React.useRef<number | undefined>(undefined);
  const dropOverRef = React.useRef(false);
  const draggingRef = React.useRef<string | null>(null);
  const [overflowing, setOverflowing] = React.useState(false);
  const [dropOver, setDropOver] = React.useState(false);

  const callbacks = React.useRef({ onScrollingChanged, onBottomChanged, onDropOverChanged, onRootDropped, onPressBlank });
  React.useEffect(() => {
    callbacks.current = { onScrollingChanged, onBottomChanged, onDropOverChanged, onRootDropped, onPressBlank };
  });

  // Scroll: scrollingChanged starts on the first scroll and ends once the scroll settles;
  // bottomChanged reports the end (bottom or right) only when it changes.
  React.useEffect(() => {
    const el = viewportRef.current;
    if (!el) return;
    const onScroll = () => {
      if (!scrollingRef.current) {
        scrollingRef.current = true;
        callbacks.current.onScrollingChanged?.(true);
      }
      window.clearTimeout(settleRef.current);
      settleRef.current = window.setTimeout(() => {
        scrollingRef.current = false;
        callbacks.current.onScrollingChanged?.(false);
      }, SETTLE_MS);
      const atEnd = horizontal
        ? el.scrollLeft + el.clientWidth >= el.scrollWidth - 1
        : el.scrollTop + el.clientHeight >= el.scrollHeight - 1;
      if (atEnd !== atBottomRef.current) {
        atBottomRef.current = atEnd;
        callbacks.current.onBottomChanged?.(atEnd);
      }
    };
    el.addEventListener("scroll", onScroll, { passive: true });
    return () => {
      el.removeEventListener("scroll", onScroll);
      window.clearTimeout(settleRef.current);
    };
  }, [horizontal, scrollbar]);

  // The region takes Tab only while its content overflows, so it can be scrolled by keyboard.
  React.useEffect(() => {
    const el = viewportRef.current;
    if (!el) return;
    const measure = () =>
      setOverflowing(horizontal ? el.scrollWidth > el.clientWidth + 1 : el.scrollHeight > el.clientHeight + 1);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    if (el.firstElementChild) observer.observe(el.firstElementChild);
    return () => observer.disconnect();
  }, [horizontal, scrollbar]);

  const setOver = (next: boolean) => {
    if (dropOverRef.current === next) return;
    dropOverRef.current = next;
    setDropOver(next);
    callbacks.current.onDropOverChanged?.(next);
  };

  // Edge auto-scroll while a row is dragged near the top or bottom of the task list.
  const autoScroll = (clientY: number) => {
    const el = viewportRef.current;
    if (!el || horizontal) return;
    const rect = el.getBoundingClientRect();
    if (clientY > rect.bottom - EDGE_PX) el.scrollTop += EDGE_STEP_PX;
    else if (clientY < rect.top + EDGE_PX) el.scrollTop -= EDGE_STEP_PX;
  };

  const onPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!isRow(e.target)) callbacks.current.onPressBlank?.();
  };

  const onDragStart = (e: React.DragEvent<HTMLDivElement>) => {
    draggingRef.current = rowId(e.target);
  };

  const onDragEnd = () => {
    draggingRef.current = null;
  };

  const onDragOver = (e: React.DragEvent<HTMLDivElement>) => {
    if (!dropTarget) return;
    autoScroll(e.clientY);
    if (isRow(e.target)) {
      setOver(false);
      return;
    }
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    setOver(true);
  };

  const onDragLeave = (e: React.DragEvent<HTMLDivElement>) => {
    if (!dropTarget) return;
    const next = e.relatedTarget;
    if (!(next instanceof Node) || !e.currentTarget.contains(next)) setOver(false);
  };

  const onDrop = (e: React.DragEvent<HTMLDivElement>) => {
    if (!dropTarget) return;
    if (isRow(e.target)) {
      setOver(false);
      return;
    }
    e.preventDefault();
    const taskId = draggingRef.current ?? e.dataTransfer?.getData("text/plain") ?? "";
    draggingRef.current = null;
    setOver(false);
    callbacks.current.onRootDropped?.(taskId);
  };

  const regionProps = {
    ref: viewportRef,
    "data-slot": "region",
    role: label ? "region" : undefined,
    "aria-label": label || undefined,
    tabIndex: overflowing ? 0 : undefined,
    "data-drop-over": dropOver ? "true" : undefined,
    className: scrollAreaViewportVariants({ overscroll, scrollbar, orientation }),
    onPointerDown,
    onDragStart,
    onDragEnd,
    onDragOver,
    onDragLeave,
    onDrop,
  };

  const content = (
    <div
      className={cn(
        "[&>*]:shrink-0",
        horizontal ? "flex h-full w-max flex-row" : "flex min-h-full flex-col",
      )}
    >
      {children}
    </div>
  );

  if (scrollbar === "hidden") {
    return (
      <div ref={ref} {...props} className={cn(scrollAreaVariants({ tone }), className)}>
        <div {...regionProps}>{content}</div>
      </div>
    );
  }

  return (
    <ScrollAreaPrimitive.Root
      ref={ref}
      type="auto"
      {...props}
      className={cn(scrollAreaVariants({ tone }), className)}
    >
      <ScrollAreaPrimitive.Viewport {...regionProps}>{content}</ScrollAreaPrimitive.Viewport>
      <ScrollAreaPrimitive.ScrollAreaScrollbar
        orientation={horizontal ? "horizontal" : "vertical"}
        className={cn(
          "flex touch-none p-px transition-colors select-none",
          horizontal ? "h-2.5 flex-col" : "h-full w-2.5",
        )}
      >
        <ScrollAreaPrimitive.ScrollAreaThumb data-slot="thumb" className="relative flex-1 rounded-full bg-edge-3" />
      </ScrollAreaPrimitive.ScrollAreaScrollbar>
    </ScrollAreaPrimitive.Root>
  );
});

export { scrollAreaVariants, scrollAreaViewportVariants };
