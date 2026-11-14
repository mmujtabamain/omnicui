import * as React from "react";
import * as PopoverPrimitive from "@radix-ui/react-popover";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { TabList, TabListItem } from "./tab-list";

/** Window: h-13 catalogue, measured 46px (1rem = 14px). Pane: the same band with an edge-1 rule. */
const titleBarVariants = cva(
  "relative z-(--z-index-raised) flex min-w-0 items-stretch border-b bg-surface-1 text-content-1 select-none",
  {
    variants: {
      scope: {
        window: "h-[3.286rem] border-edge-2",
        pane: "h-[3.286rem] border-edge-1",
      },
      scrolled: {
        true: "border-edge-3 shadow-lift",
        false: "",
      },
      drag: {
        true: "[-webkit-app-region:drag]",
        false: "",
      },
    },
    defaultVariants: { scope: "window", scrolled: false, drag: false },
  },
);

const DEMO_PROJECTS = ["omnicui", "taskscape", "docs"];
const DEMO_TABS = [
  { id: "tasks", label: "Tasks" },
  { id: "notes", label: "Notes" },
  { id: "files", label: "Files" },
  { id: "history", label: "History" },
];

/** Lit traffic lights. The spec names no yellow token, so minimize takes accent-500. */
const LIGHT_TONE = {
  close: "bg-danger-500",
  minimize: "bg-accent-500",
  maximize: "bg-success-500",
} as const;

export type TitleBarScope = "window" | "pane";
export type TitleBarControls = "leading" | "none";
export type TitleBarTitleAs = "div" | "h1" | "h2";
export type WindowControl = "close" | "minimize";

export interface TitleBarProps
  extends Omit<React.HTMLAttributes<HTMLDivElement>, "title" | "children"> {
  scope?: TitleBarScope;
  controls?: TitleBarControls;
  title?: string;
  titleAs?: TitleBarTitleAs;
  /** The project switcher label. */
  projectName?: string;
  /** Desktop window focus: lit controls while true, grey while false. */
  windowFocused?: boolean;
  /** Controlled: the project menu is open. */
  projectMenuOpen?: boolean;
  scrolled?: boolean;
  recording?: boolean;
  /** Renders the project switcher with its menu. */
  withProjectSwitcher?: boolean;
  /** Renders the list tabs. */
  withTabs?: boolean;
  /** Renders a Discard button in the trailing slot. */
  withTrailing?: boolean;
  trailing?: React.ReactNode;
  children?: React.ReactNode;
  onProjectMenuOpenChange?: (open: boolean) => void;
  /** Double-click on the drag region, or the zoom light. */
  onMaximizeToggle?: () => void;
  /** Close or minimize traffic light. */
  onWindowControl?: (control: WindowControl) => void;
}

interface TrafficLightProps {
  slot: "close" | "minimize" | "maximize";
  label: string;
  lit: boolean;
  onClick?: () => void;
}

function TrafficLight({ slot, label, lit, onClick }: TrafficLightProps) {
  return (
    <Button
      data-slot={slot}
      data-no-drag=""
      ariaLabel={label}
      variant="ghost"
      shape="round"
      size="sm"
      width="content"
      className={cn("h-[0.857rem] w-[0.857rem] p-0", lit ? LIGHT_TONE[slot] : "bg-content-3")}
      onClick={onClick}
    />
  );
}

interface WindowControlsProps {
  scope: TitleBarScope;
  lit: boolean;
  onControl?: (control: WindowControl) => void;
  onMaximize?: () => void;
}

/** Leading traffic lights. A pane shows only the close light. */
function WindowControls({ scope, lit, onControl, onMaximize }: WindowControlsProps) {
  return (
    <div
      data-slot="controls"
      data-no-drag=""
      className="flex shrink-0 items-center gap-[0.571rem] pl-[0.857rem] pr-[0.286rem]"
    >
      <TrafficLight slot="close" label="Close" lit={lit} onClick={() => onControl?.("close")} />
      {scope === "window" && (
        <>
          <TrafficLight slot="minimize" label="Minimize" lit={lit} onClick={() => onControl?.("minimize")} />
          <TrafficLight slot="maximize" label="Maximize" lit={lit} onClick={() => onMaximize?.()} />
        </>
      )}
    </div>
  );
}

export interface ProjectSwitcherProps {
  name: string;
  open: boolean;
  onOpenChange?: (open: boolean) => void;
}

/** The project switcher button and its menu. Enter, Space and Escape come from the Radix trigger. */
export function ProjectSwitcher({ name, open, onOpenChange }: ProjectSwitcherProps) {
  return (
    <PopoverPrimitive.Root open={open} onOpenChange={onOpenChange}>
      <PopoverPrimitive.Trigger asChild>
        <Button
          data-slot="projectSwitcher"
          data-no-drag=""
          variant="ghost"
          tone="strong"
          size="md"
          width="content"
          label={name}
          trailingIcon="expand_more"
          className={cn("shrink-0 self-center", open && "bg-wash-2")}
        />
      </PopoverPrimitive.Trigger>
      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          data-slot="projectMenu"
          align="start"
          sideOffset={4}
          className="z-(--z-index-popover) min-w-[11rem] rounded-(--radius-panel) border border-edge-2 bg-surface-3 p-1 text-content-1 shadow-menu animate-rise outline-none"
        >
          <div className="px-2 py-1 text-[0.786rem] font-medium text-content-3">Projects</div>
          {DEMO_PROJECTS.map((project) => (
            <PopoverPrimitive.Close asChild key={project}>
              <Button variant="ghost" size="md" block label={project} />
            </PopoverPrimitive.Close>
          ))}
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    </PopoverPrimitive.Root>
  );
}

export const TitleBar = React.forwardRef<HTMLDivElement, TitleBarProps>(function TitleBar(
  {
    scope = "window",
    controls = "none",
    title = "",
    titleAs = "div",
    projectName = "",
    windowFocused = true,
    projectMenuOpen = false,
    scrolled = false,
    recording = false,
    withProjectSwitcher = false,
    withTabs = false,
    withTrailing = false,
    trailing,
    children,
    onProjectMenuOpenChange,
    onMaximizeToggle,
    onWindowControl,
    className,
    onDoubleClick,
    ...props
  },
  ref,
) {
  const [tab, setTab] = React.useState("tasks");
  // Desktop window chrome is the window scope with leading controls. The web header has neither.
  const dragRegion = scope === "window" && controls === "leading";
  const TitleTag: TitleBarTitleAs = titleAs;

  return (
    <div
      ref={ref}
      data-slot="bar"
      data-tauri-drag-region={dragRegion ? "" : undefined}
      className={cn(titleBarVariants({ scope, scrolled, drag: dragRegion }), className)}
      onDoubleClick={(e) => {
        onDoubleClick?.(e);
        if (!dragRegion || (e.target as Element).closest("[data-no-drag]")) return;
        onMaximizeToggle?.();
      }}
      {...props}
    >
      {controls === "leading" && (
        <WindowControls
          scope={scope}
          lit={windowFocused}
          onControl={onWindowControl}
          onMaximize={onMaximizeToggle}
        />
      )}

      <div
        data-slot="contentSlot"
        className="flex min-w-0 flex-1 items-stretch gap-[0.429rem] px-[0.714rem]"
      >
        {withProjectSwitcher && (
          <ProjectSwitcher
            name={projectName || "Project"}
            open={projectMenuOpen}
            onOpenChange={onProjectMenuOpenChange}
          />
        )}

        <TitleTag
          data-slot="title"
          className="flex min-w-0 shrink items-center self-center text-[0.929rem] leading-none font-medium text-content-1"
        >
          <span className="truncate">{title}</span>
        </TitleTag>

        {withTabs && (
          <div data-slot="tabs" data-no-drag="" className="flex min-w-0 flex-1 items-stretch">
            <TabList label="Views" value={tab} onValueChange={setTab}>
              {DEMO_TABS.map((item) => (
                <TabListItem key={item.id} value={item.id}>
                  {item.label}
                </TabListItem>
              ))}
            </TabList>
          </div>
        )}

        {children}
      </div>

      {recording && (
        <span
          data-slot="recordingStatus"
          role="status"
          className="flex shrink-0 items-center gap-[0.429rem] self-center px-2 text-[0.786rem] leading-none font-medium text-danger-500"
        >
          <span aria-hidden="true" className="size-[0.429rem] rounded-full bg-danger-500" />
          Recording
        </span>
      )}

      {(withTrailing || trailing) && (
        <div data-no-drag="" className="flex shrink-0 items-center gap-2 pr-[0.714rem]">
          {withTrailing && (
            <Button
              data-slot="trailing"
              variant="outline"
              tone="strong"
              width="content"
              label="Discard"
            />
          )}
          {trailing}
        </div>
      )}
    </div>
  );
});

export { titleBarVariants };
