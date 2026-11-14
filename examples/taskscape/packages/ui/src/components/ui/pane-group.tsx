import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";

export type PaneGroupDirection = "row" | "column";
export type PaneGroupLayout = { task: number; split: number; aside: number };

/* Percent floors, the aside's pixel band (row only), and the key and press rules of the contract. */
const TASK_MIN = 25;
const SPLIT_MIN = 20;
const ASIDE_MIN_PX = 280;
const ASIDE_MAX_PX = 420;
const NARROW_PX = 600;
const STEP = 5;
const BIG_STEP = 10;
const PRESS_MS = 500;

export const paneGroupVariants = cva(
  "relative h-full w-full min-h-0 min-w-0 flex-1 select-none overflow-hidden bg-surface-1",
  {
    variants: {
      direction: { row: "", column: "" },
    },
    defaultVariants: { direction: "row" },
  },
);

export const paneGroupPaneVariants = cva("absolute min-h-0 min-w-0 overflow-hidden", {
  variants: {
    kind: {
      task: "bg-surface-2",
      split: "bg-surface-2",
      aside: "bg-surface-1",
    },
  },
  defaultVariants: { kind: "task" },
});

export const paneGroupResizerVariants = cva("group/resizer absolute z-(--z-index-raised) touch-none outline-none focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-inset", {
  variants: {
    direction: {
      row: "inset-y-0 w-[0.5rem] -translate-x-1/2 cursor-col-resize",
      column: "inset-x-0 h-[0.5rem] -translate-y-1/2 cursor-row-resize",
    },
  },
  defaultVariants: { direction: "row" },
});

type Geom = { row: boolean; length: number; hasSplit: boolean; asideVisible: boolean };
type Stored = { topo: string; task: number; aside: number; collapsed: boolean };
type Sizes = Pick<Stored, "task" | "aside" | "collapsed">;
type Key = -1 | 1 | "home" | "end";
type Drag = {
  kind: "task" | "aside";
  row: boolean;
  x: number;
  y: number;
  time: number;
  base: Stored;
  geom: Geom;
  start: PaneGroupLayout;
  moved: boolean;
  next: Stored | null;
  layout: PaneGroupLayout | null;
};

const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), hi);
const round = (v: number) => Math.round(v * 100) / 100;
const same = (a: PaneGroupLayout, b: PaneGroupLayout) =>
  round(a.task) === round(b.task) && round(a.split) === round(b.split) && round(a.aside) === round(b.aside);
const publicLayout = (l: PaneGroupLayout): PaneGroupLayout => ({
  task: round(l.task),
  split: round(l.split),
  aside: round(l.aside),
});

/** Aside bounds in percent of the group length: the 280 to 420 px band (row), capped so the other panes keep their floors. */
function asideRange(g: Geom, task0: number): [number, number] {
  const cap = Math.max(0, g.hasSplit ? 100 - task0 - SPLIT_MIN : 100 - TASK_MIN);
  if (!g.row || g.length <= 0) return [0, cap];
  return [
    Math.min((ASIDE_MIN_PX * 100) / g.length, cap),
    Math.min((ASIDE_MAX_PX * 100) / g.length, cap),
  ];
}

/** The displayed sizes for stored sizes at the current geometry. Every limit is re-applied here. */
function resolve(s: Sizes, g: Geom): PaneGroupLayout {
  if (!g.hasSplit) {
    const [lo, hi] = g.asideVisible ? asideRange(g, 0) : [0, 0];
    const aside = clamp(s.aside, lo, hi);
    return { task: 100 - aside, split: 0, aside };
  }
  const task0 = s.collapsed ? 0 : s.task;
  const [lo, hi] = g.asideVisible ? asideRange(g, task0) : [0, 0];
  const aside = clamp(s.aside, lo, hi);
  if (s.collapsed) return { task: 0, split: 100 - aside, aside };
  const task = clamp(task0, TASK_MIN, 100 - aside - SPLIT_MIN);
  return { task, split: 100 - task - aside, aside };
}

/** The stored sizes after a user change. The aside keeps its size while it is hidden by width. */
function storeFrom(base: Stored, raw: Sizes, g: Geom): Stored {
  const r = resolve(raw, g);
  return {
    topo: base.topo,
    task: g.hasSplit ? (raw.collapsed ? raw.task : r.task) : base.task,
    aside: g.asideVisible ? r.aside : base.aside,
    collapsed: g.hasSplit && raw.collapsed,
  };
}

/** Default sizes: first layout and reset (contract). */
function defaultsFor(g: Geom): { task: number; aside: number } {
  if (g.hasSplit) return g.asideVisible ? { task: 40, aside: 30 } : { task: 50, aside: 0 };
  return g.asideVisible ? { task: 70, aside: 30 } : { task: 100, aside: 0 };
}

function keyOf(key: string, row: boolean): Key | null {
  if (key === "Home") return "home";
  if (key === "End") return "end";
  if (key === (row ? "ArrowLeft" : "ArrowUp")) return -1;
  if (key === (row ? "ArrowRight" : "ArrowDown")) return 1;
  return null;
}

const storageKey = (id: string) => `taskscape.panegroup.${id}`;

function loadStored(id: string, topo: string): Stored | null {
  if (!id || typeof window === "undefined") return null;
  try {
    const v = JSON.parse(window.localStorage.getItem(storageKey(id)) ?? "null");
    if (v && v.topo === topo && typeof v.task === "number" && typeof v.aside === "number")
      return { topo, task: v.task, aside: v.aside, collapsed: v.collapsed === true };
  } catch {
    /* Unreadable saved sizes: the defaults are used. */
  }
  return null;
}

function saveStored(id: string, s: Stored) {
  if (!id || typeof window === "undefined") return;
  try {
    window.localStorage.setItem(storageKey(id), JSON.stringify(s));
  } catch {
    /* Storage blocked or full: the sizes are not kept. */
  }
}

function geometry(row: boolean, start: number, size: number): React.CSSProperties {
  return row
    ? { left: `${start}%`, width: `${size}%`, top: 0, bottom: 0 }
    : { top: `${start}%`, height: `${size}%`, left: 0, right: 0 };
}

type ResizerProps = React.HTMLAttributes<HTMLDivElement> & {
  slot: string;
  label: string;
  direction: PaneGroupDirection;
  position: number;
  value: number;
  min: number;
  max: number;
};

/** A focusable window splitter (role separator, APG). Its keys and pointer come from PaneGroup. */
function Resizer({ slot, label, direction, position, value, min, max, className, ...handlers }: ResizerProps) {
  const row = direction === "row";
  return (
    <div
      data-slot={slot}
      role="separator"
      tabIndex={0}
      aria-label={label}
      aria-orientation={row ? "vertical" : "horizontal"}
      aria-valuenow={value}
      aria-valuemin={min}
      aria-valuemax={max}
      className={cn(paneGroupResizerVariants({ direction }), className)}
      style={row ? { left: `${position}%` } : { top: `${position}%` }}
      {...handlers}
    >
      <span
        aria-hidden
        className={cn(
          "pointer-events-none absolute bg-edge-2 transition-colors duration-(--duration-fast) ease-standard group-hover/resizer:bg-accent-500 group-focus-visible/resizer:bg-focus-1",
          row ? "inset-y-0 left-1/2 w-px" : "inset-x-0 top-1/2 h-px",
        )}
      />
    </div>
  );
}

export interface PaneGroupProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "children"> {
  direction?: PaneGroupDirection;
  /** Shows the split pane and the task handle. */
  split?: boolean;
  /** Hides the aside and its handle; the other panes take the space. */
  asideHidden?: boolean;
  /** Only the task pane, at full size. */
  noLists?: boolean;
  /** Key under which the sizes are kept on this device. "" keeps nothing. */
  autoSaveId?: string;
  /** Fires once per user change that moves the sizes. */
  onLayoutChanged?: (layout: PaneGroupLayout) => void;
  taskContent?: React.ReactNode;
  splitContent?: React.ReactNode;
  asideContent?: React.ReactNode;
  taskLabel?: string;
  splitLabel?: string;
  asideLabel?: string;
}

export const PaneGroup = React.forwardRef<HTMLDivElement, PaneGroupProps>(function PaneGroup(
  {
    direction = "row",
    split = false,
    asideHidden = false,
    noLists = false,
    autoSaveId = "",
    onLayoutChanged,
    taskContent,
    splitContent,
    asideContent,
    taskLabel = "Tasks",
    splitLabel = "Split pane",
    asideLabel = "Preview",
    className,
    ...props
  },
  ref,
) {
  const rootRef = React.useRef<HTMLDivElement | null>(null);
  const dragRef = React.useRef<Drag | null>(null);
  const pressRef = React.useRef<{ kind: "task" | "aside"; time: number } | null>(null);
  const [size, setSize] = React.useState({ width: 0, height: 0 });

  const setRoot = React.useCallback(
    (node: HTMLDivElement | null) => {
      rootRef.current = node;
      if (typeof ref === "function") ref(node);
      else if (ref) (ref as React.MutableRefObject<HTMLDivElement | null>).current = node;
    },
    [ref],
  );

  React.useLayoutEffect(() => {
    const el = rootRef.current;
    if (!el) return;
    const measure = () => {
      const r = el.getBoundingClientRect();
      setSize((prev) => (prev.width === r.width && prev.height === r.height ? prev : { width: r.width, height: r.height }));
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const row = direction === "row";
  const hasSplit = split && !noLists;
  const asideProp = !asideHidden && !noLists;
  const geom: Geom = {
    row,
    length: row ? size.width : size.height,
    hasSplit,
    asideVisible: asideProp && (!row || size.width >= NARROW_PX),
  };
  const topo = `${direction}|${hasSplit}|${asideProp}`;
  const [stored, setStored] = React.useState<Stored | null>(() => loadStored(autoSaveId, topo));
  const base: Stored = stored && stored.topo === topo ? stored : { topo, ...defaultsFor(geom), collapsed: false };
  const layout = resolve(base, geom);
  const collapsed = hasSplit && base.collapsed;
  const task = layout.task;
  const mid = layout.split;
  const aside = layout.aside;
  const [asideLo, asideHi] = geom.asideVisible ? asideRange(geom, collapsed ? 0 : base.task) : [0, 0];

  const commit = (raw: Sizes) => {
    const next = storeFrom(base, raw, geom);
    setStored(next);
    saveStored(autoSaveId, next);
    const after = resolve(next, geom);
    if (!same(after, layout)) onLayoutChanged?.(publicLayout(after));
  };

  const resetLayout = () => {
    const d = defaultsFor(geom);
    commit({ task: d.task, aside: d.aside, collapsed: false });
  };

  const onTaskKey = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      commit(collapsed ? { task: base.task, aside, collapsed: false } : { task, aside, collapsed: true });
      return;
    }
    const dir = keyOf(e.key, row);
    if (dir === null || collapsed) return;
    e.preventDefault();
    const amount = e.shiftKey ? BIG_STEP : STEP;
    const target = dir === "home" ? 0 : dir === "end" ? 100 : task + dir * amount;
    commit({ task: target, aside, collapsed: false });
  };

  const onAsideKey = (e: React.KeyboardEvent<HTMLDivElement>) => {
    const dir = keyOf(e.key, row);
    if (dir === null) return;
    e.preventDefault();
    const amount = e.shiftKey ? BIG_STEP : STEP;
    // The handle's position is 100 minus the aside, so a step toward the end shrinks the aside.
    const target = dir === "home" ? 100 : dir === "end" ? 0 : aside - dir * amount;
    commit({ task: base.task, aside: target, collapsed });
  };

  const onPointerDown = (kind: "task" | "aside") => (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    const now = Date.now();
    const press = pressRef.current;
    if (e.detail >= 2 || (press && press.kind === kind && now - press.time < PRESS_MS)) {
      pressRef.current = null;
      dragRef.current = null;
      resetLayout();
      return;
    }
    e.currentTarget.setPointerCapture(e.pointerId);
    dragRef.current = {
      kind,
      row,
      x: e.clientX,
      y: e.clientY,
      time: now,
      base,
      geom,
      start: layout,
      moved: false,
      next: null,
      layout: null,
    };
  };

  const onPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    const d = dragRef.current;
    if (!d) return;
    const delta = d.row ? e.clientX - d.x : e.clientY - d.y;
    if (delta !== 0) d.moved = true;
    const pct = d.geom.length > 0 ? (delta / d.geom.length) * 100 : 0;
    const raw: Sizes =
      d.kind === "task"
        ? { task: d.start.task + pct, aside: d.start.aside, collapsed: false }
        : { task: d.base.task, aside: d.start.aside - pct, collapsed: d.base.collapsed };
    const next = storeFrom(d.base, raw, d.geom);
    d.next = next;
    d.layout = resolve(next, d.geom);
    setStored(next);
  };

  const onPointerUp = () => {
    const d = dragRef.current;
    dragRef.current = null;
    if (!d) return;
    if (!d.moved) {
      pressRef.current = { kind: d.kind, time: d.time };
      return;
    }
    if (!d.next || !d.layout) return;
    saveStored(autoSaveId, d.next);
    if (!same(d.layout, d.start)) onLayoutChanged?.(publicLayout(d.layout));
  };

  return (
    <div
      ref={setRoot}
      data-direction={direction}
      className={cn(paneGroupVariants({ direction }), className)}
      {...props}
    >
      <div
        data-slot="taskPane"
        role="region"
        aria-label={taskLabel}
        hidden={collapsed}
        className={paneGroupPaneVariants({ kind: "task" })}
        style={geometry(row, 0, task)}
      >
        {taskContent}
      </div>
      {hasSplit && (
        <div
          data-slot="splitPane"
          role="region"
          aria-label={splitLabel}
          className={paneGroupPaneVariants({ kind: "split" })}
          style={geometry(row, task, mid)}
        >
          {splitContent}
        </div>
      )}
      {geom.asideVisible && (
        <div
          data-slot="aside"
          role="complementary"
          aria-label={asideLabel}
          className={paneGroupPaneVariants({ kind: "aside" })}
          style={geometry(row, 100 - aside, aside)}
        >
          {asideContent}
        </div>
      )}
      {hasSplit && (
        <Resizer
          slot="taskResizer"
          label="Resize panes"
          direction={direction}
          position={task}
          value={round(task)}
          min={collapsed ? 0 : TASK_MIN}
          max={round(100 - aside - SPLIT_MIN)}
          onKeyDown={onTaskKey}
          onPointerDown={onPointerDown("task")}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerUp}
        />
      )}
      {geom.asideVisible && (
        <Resizer
          slot="asideResizer"
          label="Resize preview"
          direction={direction}
          position={100 - aside}
          value={round(100 - aside)}
          min={round(100 - asideHi)}
          max={round(100 - asideLo)}
          onKeyDown={onAsideKey}
          onPointerDown={onPointerDown("aside")}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerUp}
        />
      )}
    </div>
  );
});
