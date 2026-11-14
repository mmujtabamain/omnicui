import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";

export type PaneGroupDirection = "row" | "column";
export type PaneKind = "task" | "aside";

/** Limits from the contract. Sizes are percent of the group length; the aside is in px in a row. */
const TASK_MIN = 25;
const SPLIT_MIN = 20;
const ASIDE_MIN_PX = 280;
const ASIDE_MAX_PX = 420;
const ASIDE_SHOW_PX = 600;
const ASIDE_DEFAULT = 30;
/** Two presses on one handle this close, with no drag between, reset the sizes. */
const DOUBLE_PRESS_MS = 500;

export const paneGroupVariants = cva("relative flex min-h-0 min-w-0 flex-1 overflow-hidden bg-surface-1", {
  variants: {
    direction: {
      row: "flex-row",
      column: "flex-col",
    },
  },
  defaultVariants: { direction: "row" },
});

export const paneVariants = cva("flex min-h-0 min-w-0 shrink-0 grow-0 flex-col overflow-hidden", {
  variants: {
    tone: {
      content: "bg-surface-2",
      recessed: "bg-surface-1",
    },
  },
  defaultVariants: { tone: "content" },
});

/** The separator: a hit area straddling the pane edge. */
export const paneResizerVariants = cva(
  "group/resizer absolute outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1",
  {
    variants: {
      direction: {
        row: "inset-y-0 -left-[0.25rem] w-[0.5rem] cursor-col-resize",
        column: "inset-x-0 -top-[0.25rem] h-[0.5rem] cursor-row-resize",
      },
    },
    defaultVariants: { direction: "row" },
  },
);

/** The 1px line drawn inside the separator. */
export const paneResizerLineVariants = cva(
  "pointer-events-none absolute bg-edge-2 transition-colors duration-(--duration-fast) ease-standard group-hover/resizer:bg-accent-500 group-focus-visible/resizer:bg-focus-1 group-data-[dragging=true]/resizer:bg-accent-500",
  {
    variants: {
      direction: {
        row: "inset-y-0 left-1/2 w-px -translate-x-1/2",
        column: "inset-x-0 top-1/2 h-px -translate-y-1/2",
      },
    },
    defaultVariants: { direction: "row" },
  },
);

/** The stored layout. null means the default for the current configuration. */
interface Stored {
  task: number | null;
  aside: number | null;
  /** The task size before Enter collapsed it; null when expanded. */
  collapsed: number | null;
}

interface Sizes {
  task: number;
  split: number;
  aside: number;
}

interface Cfg {
  direction: PaneGroupDirection;
  length: number;
  hasSplit: boolean;
  hasAside: boolean;
}

interface Range {
  valueNow: number;
  min: number;
  max: number;
}

export interface PaneGroupLayout {
  task: number;
  split: number;
  aside: number;
}

const EMPTY: Stored = { task: null, aside: null, collapsed: null };

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
const round = (v: number) => Math.round(v * 1000) / 1000;
const sameSizes = (a: Sizes, b: Sizes) => Math.abs(a.task - b.task) < 1e-6 && Math.abs(a.aside - b.aside) < 1e-6;

/** Aside size bounds in percent. Column has no pixel limits; the task and split floors always apply. */
function asideBounds(cfg: Cfg): [number, number] {
  const ceiling = 100 - TASK_MIN - (cfg.hasSplit ? SPLIT_MIN : 0);
  if (cfg.direction === "column" || cfg.length <= 0) return [0, ceiling];
  return [(ASIDE_MIN_PX / cfg.length) * 100, Math.min(ceiling, (ASIDE_MAX_PX / cfg.length) * 100)];
}

/** The aside handle's position (percent from the start edge) and its reach, given the task size. */
function asideReach(cfg: Cfg, task: number) {
  const [lo, hi] = asideBounds(cfg);
  const top = cfg.hasSplit ? Math.min(hi, 100 - task - SPLIT_MIN) : hi;
  return { minPos: 100 - top, maxPos: 100 - lo };
}

/** The task size's reach, given the aside size. */
function taskReach(aside: number) {
  return { lo: TASK_MIN, hi: 100 - aside - SPLIT_MIN };
}

/** Turns the stored layout into sizes for the current group length and visible panes. */
function resolveSizes(cfg: Cfg, s: Stored): Sizes {
  let aside = 0;
  if (cfg.hasAside) {
    const [lo, hi] = asideBounds(cfg);
    aside = clamp(s.aside ?? ASIDE_DEFAULT, lo, hi);
  }
  const rest = 100 - aside;
  if (!cfg.hasSplit) return { task: rest, split: 0, aside };
  if (s.collapsed !== null) return { task: 0, split: rest, aside };
  const task = clamp(s.task ?? (cfg.hasAside ? 40 : 50), TASK_MIN, rest - SPLIT_MIN);
  return { task, split: rest - task, aside };
}

function stepTask(cfg: Cfg, cur: Sizes, base: Stored, target: number): Stored {
  const { lo, hi } = taskReach(cur.aside);
  return { task: clamp(target, lo, hi), aside: cfg.hasAside ? cur.aside : base.aside, collapsed: null };
}

function stepAside(cfg: Cfg, cur: Sizes, base: Stored, target: number): Stored {
  const { minPos, maxPos } = asideReach(cfg, cur.task);
  return {
    task: cfg.hasSplit ? cur.task : base.task,
    aside: 100 - clamp(target, minPos, maxPos),
    collapsed: base.collapsed,
  };
}

function readStored(key: string): Stored {
  if (!key) return EMPTY;
  try {
    const parsed: unknown = JSON.parse(window.localStorage.getItem(key) ?? "null");
    if (!parsed || typeof parsed !== "object") return EMPTY;
    const p = parsed as Record<string, unknown>;
    const num = (v: unknown) => (typeof v === "number" && Number.isFinite(v) ? v : null);
    return { task: num(p.task), aside: num(p.aside), collapsed: num(p.collapsed) };
  } catch {
    return EMPTY;
  }
}

interface PaneGroupContextValue {
  direction: PaneGroupDirection;
  sizes: Sizes;
  hasSplit: boolean;
  hasAside: boolean;
  collapsed: boolean;
  dragKind: PaneKind | null;
  ranges: Record<PaneKind, Range>;
  onKeyDown: (kind: PaneKind, e: React.KeyboardEvent<HTMLElement>) => void;
  onPointerDown: (kind: PaneKind, e: React.PointerEvent<HTMLElement>) => void;
  onPointerMove: (kind: PaneKind, e: React.PointerEvent<HTMLElement>) => void;
  onPointerEnd: (kind: PaneKind, e: React.PointerEvent<HTMLElement>) => void;
}

const PaneGroupContext = React.createContext<PaneGroupContextValue | null>(null);

function usePaneGroup(part: string): PaneGroupContextValue {
  const ctx = React.useContext(PaneGroupContext);
  if (!ctx) throw new Error(`${part} must be rendered inside PaneGroup`);
  return ctx;
}

export interface PaneGroupProps extends React.HTMLAttributes<HTMLDivElement> {
  /** "row": panes side by side, pixel limits on the aside. "column": stacked, percent limits only. */
  direction?: PaneGroupDirection;
  /** Shows the split pane and the task handle. */
  split?: boolean;
  /** Hides the aside and the aside handle. */
  asideHidden?: boolean;
  /** Only the task pane, at full size. */
  noLists?: boolean;
  /** Non-empty: sizes are kept on this device under this key. */
  autoSaveId?: string;
  /** Fires once per user change that moves sizes: a drag release, a key step, a collapse or restore, a reset. */
  onLayoutChanged?: (layout: PaneGroupLayout) => void;
}

export const PaneGroup = React.forwardRef<HTMLDivElement, PaneGroupProps>(function PaneGroup(
  {
    direction: directionProp = "row",
    split = false,
    asideHidden = false,
    noLists = false,
    autoSaveId = "",
    onLayoutChanged,
    className,
    children,
    ...props
  },
  forwardedRef,
) {
  const direction: PaneGroupDirection = directionProp === "column" ? "column" : "row";
  const storageKey = autoSaveId ? `pane-group:${autoSaveId}` : "";
  const rootRef = React.useRef<HTMLDivElement | null>(null);
  const [length, setLength] = React.useState(0);
  const [stored, setStored] = React.useState<Stored>(() => readStored(storageKey));
  const [live, setLive] = React.useState<Stored | null>(null);
  const [dragKind, setDragKind] = React.useState<PaneKind | null>(null);
  const drag = React.useRef<{ kind: PaneKind; x: number; y: number; start: Sizes; base: Stored; draft: Stored } | null>(
    null,
  );
  const presses = React.useRef<Partial<Record<PaneKind, { time: number; moved: boolean }>>>({});

  React.useLayoutEffect(() => {
    const el = rootRef.current;
    if (!el) return;
    const measure = () => {
      const box = el.getBoundingClientRect();
      setLength(direction === "row" ? box.width : box.height);
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, [direction]);

  const hasSplit = split && !noLists;
  const hasAside = !asideHidden && !noLists && (direction === "column" || length >= ASIDE_SHOW_PX);
  const cfg: Cfg = { direction, length, hasSplit, hasAside };
  const view = live ?? stored;
  const sizes = resolveSizes(cfg, view);
  const collapsed = hasSplit && view.collapsed !== null;

  const commit = (next: Stored) => {
    setStored(next);
    setLive(null);
    setDragKind(null);
    if (storageKey) {
      try {
        window.localStorage.setItem(storageKey, JSON.stringify(next));
      } catch {
        /* storage blocked or full: the layout still works for this session */
      }
    }
  };

  const report = (next: Stored) => {
    commit(next);
    const s = resolveSizes(cfg, next);
    onLayoutChanged?.({ task: round(s.task), split: round(s.split), aside: round(s.aside) });
  };

  const onKeyDown = (kind: PaneKind, e: React.KeyboardEvent<HTMLElement>) => {
    const row = direction === "row";
    if (kind === "task" && e.key === "Enter") {
      e.preventDefault();
      const aside = hasAside ? sizes.aside : stored.aside;
      report(
        stored.collapsed !== null
          ? { task: stored.collapsed, aside, collapsed: null }
          : { task: sizes.task, aside, collapsed: sizes.task },
      );
      return;
    }
    const amount = e.shiftKey ? 10 : 5;
    const back = row ? "ArrowLeft" : "ArrowUp";
    const forward = row ? "ArrowRight" : "ArrowDown";
    let next: Stored | null = null;
    if (kind === "task") {
      const { hi } = taskReach(sizes.aside);
      const target =
        e.key === "Home" ? TASK_MIN
        : e.key === "End" ? hi
        : e.key === back ? sizes.task - amount
        : e.key === forward ? sizes.task + amount
        : null;
      if (target !== null) next = stepTask(cfg, sizes, stored, target);
    } else {
      const { minPos, maxPos } = asideReach(cfg, sizes.task);
      const pos = 100 - sizes.aside;
      const target =
        e.key === "Home" ? minPos
        : e.key === "End" ? maxPos
        : e.key === back ? pos - amount
        : e.key === forward ? pos + amount
        : null;
      if (target !== null) next = stepAside(cfg, sizes, stored, target);
    }
    if (!next) return;
    e.preventDefault();
    if (!sameSizes(resolveSizes(cfg, next), sizes)) report(next);
  };

  const onPointerDown = (kind: PaneKind, e: React.PointerEvent<HTMLElement>) => {
    if (e.button !== 0) return;
    const now = performance.now();
    const prev = presses.current[kind];
    if (prev && !prev.moved && now - prev.time < DOUBLE_PRESS_MS) {
      presses.current[kind] = undefined;
      report(EMPTY);
      return;
    }
    presses.current[kind] = { time: now, moved: false };
    e.currentTarget.setPointerCapture(e.pointerId);
    drag.current = { kind, x: e.clientX, y: e.clientY, start: sizes, base: stored, draft: stored };
  };

  const onPointerMove = (kind: PaneKind, e: React.PointerEvent<HTMLElement>) => {
    const d = drag.current;
    if (!d || d.kind !== kind || cfg.length <= 0) return;
    const px = direction === "row" ? e.clientX - d.x : e.clientY - d.y;
    if (px === 0) return;
    const press = presses.current[kind];
    if (press) press.moved = true;
    const pct = (px / cfg.length) * 100;
    const draft =
      kind === "task"
        ? stepTask(cfg, d.start, d.base, d.start.task + pct)
        : stepAside(cfg, d.start, d.base, 100 - d.start.aside + pct);
    d.draft = draft;
    setLive(draft);
    setDragKind(kind);
  };

  const onPointerEnd = (kind: PaneKind, e: React.PointerEvent<HTMLElement>) => {
    const d = drag.current;
    if (!d || d.kind !== kind) return;
    drag.current = null;
    if (e.currentTarget.hasPointerCapture(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
    if (sameSizes(resolveSizes(cfg, d.draft), d.start)) {
      setLive(null);
      setDragKind(null);
      return;
    }
    report(d.draft);
  };

  const taskHi = taskReach(sizes.aside).hi;
  const asideR = asideReach(cfg, sizes.task);
  const ranges: Record<PaneKind, Range> = {
    task: { valueNow: sizes.task, min: collapsed ? 0 : TASK_MIN, max: taskHi },
    aside: { valueNow: 100 - sizes.aside, min: asideR.minPos, max: asideR.maxPos },
  };

  const ctx: PaneGroupContextValue = {
    direction,
    sizes,
    hasSplit,
    hasAside,
    collapsed,
    dragKind,
    ranges,
    onKeyDown,
    onPointerDown,
    onPointerMove,
    onPointerEnd,
  };

  const setRootRef = React.useCallback(
    (node: HTMLDivElement | null) => {
      rootRef.current = node;
      if (typeof forwardedRef === "function") forwardedRef(node);
      else if (forwardedRef) forwardedRef.current = node;
    },
    [forwardedRef],
  );

  return (
    <PaneGroupContext.Provider value={ctx}>
      <div
        ref={setRootRef}
        data-direction={direction}
        className={cn(paneGroupVariants({ direction }), dragKind !== null && "select-none", className)}
        {...props}
      >
        {children}
      </div>
    </PaneGroupContext.Provider>
  );
});

export interface PaneGroupPaneProps extends React.HTMLAttributes<HTMLElement> {
  /** The pane's heading; the region is labelled by it. */
  heading: string;
}

interface PaneRegionProps extends PaneGroupPaneProps {
  slot: string;
  basis: number;
  tone: "content" | "recessed";
  as?: "section" | "aside";
  collapsed?: boolean;
}

function PaneRegion({ slot, basis, tone, as = "section", collapsed = false, heading, className, style, children, ...props }: PaneRegionProps) {
  const id = React.useId();
  const Comp = as;
  return (
    <Comp
      {...props}
      aria-labelledby={id}
      data-slot={slot}
      className={cn(paneVariants({ tone }), collapsed && "hidden", className)}
      style={{ ...style, flexBasis: `${basis}%` }}
    >
      <h2
        id={id}
        className="truncate border-b border-edge-1 px-3 py-2 text-[0.857rem] leading-[1.143rem] font-semibold text-content-2"
      >
        {heading}
      </h2>
      <div className="flex min-h-0 min-w-0 flex-1 flex-col overflow-auto">{children}</div>
    </Comp>
  );
}

/** The task pane. Renders the task handle after itself when the split pane is shown. */
export function PaneGroupTask(props: PaneGroupPaneProps) {
  const ctx = usePaneGroup("PaneGroupTask");
  return (
    <>
      <PaneRegion {...props} slot="taskPane" tone="content" basis={ctx.sizes.task} collapsed={ctx.collapsed} />
      {ctx.hasSplit && <PaneGroupResizer kind="task" />}
    </>
  );
}

/** The split pane, shown only when split is set and lists are on. */
export function PaneGroupSplit(props: PaneGroupPaneProps) {
  const ctx = usePaneGroup("PaneGroupSplit");
  if (!ctx.hasSplit) return null;
  return <PaneRegion {...props} slot="splitPane" tone="content" basis={ctx.sizes.split} />;
}

/** The aside. Renders the aside handle before itself when the aside is shown. */
export function PaneGroupAside(props: PaneGroupPaneProps) {
  const ctx = usePaneGroup("PaneGroupAside");
  if (!ctx.hasAside) return null;
  return (
    <>
      <PaneGroupResizer kind="aside" />
      <PaneRegion {...props} slot="aside" as="aside" tone="recessed" basis={ctx.sizes.aside} />
    </>
  );
}

/** A window splitter: a focusable separator that drags and steps the pane sizes. */
export function PaneGroupResizer({ kind }: { kind: PaneKind }) {
  const ctx = usePaneGroup("PaneGroupResizer");
  const range = ctx.ranges[kind];
  const row = ctx.direction === "row";
  return (
    <div className={cn("relative z-(--z-index-raised) shrink-0 grow-0", row ? "h-full w-0" : "h-0 w-full")}>
      <div
        role="separator"
        tabIndex={0}
        data-slot={kind === "task" ? "taskResizer" : "asideResizer"}
        aria-label={kind === "task" ? "Resize panes" : "Resize preview"}
        aria-orientation={row ? "vertical" : "horizontal"}
        aria-valuenow={round(range.valueNow)}
        aria-valuemin={round(range.min)}
        aria-valuemax={round(range.max)}
        data-dragging={ctx.dragKind === kind ? "true" : undefined}
        className={paneResizerVariants({ direction: ctx.direction })}
        onKeyDown={(e) => ctx.onKeyDown(kind, e)}
        onPointerDown={(e) => ctx.onPointerDown(kind, e)}
        onPointerMove={(e) => ctx.onPointerMove(kind, e)}
        onPointerUp={(e) => ctx.onPointerEnd(kind, e)}
        onPointerCancel={(e) => ctx.onPointerEnd(kind, e)}
      >
        <span aria-hidden className={paneResizerLineVariants({ direction: ctx.direction })} />
      </div>
    </div>
  );
}
