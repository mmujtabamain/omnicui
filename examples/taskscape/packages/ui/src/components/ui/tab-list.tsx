import * as React from "react";
import * as TabsPrimitive from "@radix-ui/react-tabs";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";

type Orientation = "horizontal" | "vertical";
type Overflow = "scroll" | "fit";

/** The strip: a flex row in the title bar, or a column in the Settings sidebar. Scrollbar hidden. */
const tabListVariants = cva("[scrollbar-width:none] [&::-webkit-scrollbar]:hidden", {
  variants: {
    orientation: {
      horizontal: "flex h-full min-w-0 flex-1 items-stretch [-webkit-app-region:no-drag]",
      vertical: "flex w-full flex-col gap-1 p-[0.143rem]",
    },
    overflow: {
      scroll: "",
      fit: "overflow-hidden",
    },
  },
  compoundVariants: [
    { orientation: "horizontal", overflow: "scroll", className: "overflow-x-auto" },
    { orientation: "vertical", overflow: "scroll", className: "overflow-y-auto" },
  ],
  defaultVariants: { orientation: "horizontal", overflow: "scroll" },
});

/** Selected is the data-state the Radix trigger sets: a 2px accent rule under a title-bar tab, a wash behind a settings row. */
const tabListItemVariants = cva(
  [
    "group/tab relative inline-flex min-w-0 select-none items-center gap-1.5 whitespace-nowrap font-medium outline-none",
    "text-content-2 transition-[color,background-color] duration-(--duration-fast) ease-standard hover:text-content-1",
    "focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1",
    "disabled:pointer-events-none disabled:opacity-45",
    "data-[state=active]:text-content-1",
  ],
  {
    variants: {
      orientation: {
        horizontal:
          "h-full justify-center px-3 text-[0.929rem] after:absolute after:inset-x-2 after:bottom-0 after:h-[0.143rem] after:rounded-full after:bg-transparent data-[state=active]:after:bg-accent-500",
        vertical:
          "h-[2rem] w-full shrink-0 justify-start rounded-(--radius-field) px-2 text-[0.929rem] hover:bg-wash-1 data-[state=active]:bg-wash-2",
      },
      overflow: {
        scroll: "",
        fit: "",
      },
    },
    compoundVariants: [
      { orientation: "horizontal", overflow: "scroll", className: "shrink-0" },
      { orientation: "horizontal", overflow: "fit", className: "min-w-0 flex-1 shrink truncate" },
    ],
    defaultVariants: { orientation: "horizontal", overflow: "scroll" },
  },
);

const TabListContext = React.createContext<{ orientation: Orientation; overflow: Overflow }>({
  orientation: "horizontal",
  overflow: "scroll",
});

export interface TabListProps
  extends Omit<React.ComponentPropsWithoutRef<typeof TabsPrimitive.List>, "loop" | "orientation"> {
  /** Selected tab id; "" selects none. */
  value?: string;
  /** The newly selected tab id. */
  onValueChange?: (value: string) => void;
  /** true: arrows wrap past the first and last tab. */
  loop?: boolean;
  /** aria-label of the tablist. */
  label?: string;
  orientation?: Orientation;
  overflow?: Overflow;
}

export type TabListItemProps = React.ComponentPropsWithoutRef<typeof TabsPrimitive.Trigger>;

/** Part name for the tab at `index` of `count`: tab 1, tab 2 and the last tab. */
export function tabSlot(index: number, count: number): string {
  if (index === 0) return "firstTab";
  if (index === count - 1) return "lastTab";
  if (index === 1) return "secondTab";
  return "tab";
}

export const TabList = React.forwardRef<HTMLDivElement, TabListProps>(function TabList(
  {
    value = "1",
    onValueChange,
    loop = true,
    label = "Tabs",
    orientation = "horizontal",
    overflow = "scroll",
    className,
    children,
    ...props
  },
  ref,
) {
  // With no tabs the list is generic: it has no tab role to report.
  const empty = React.Children.count(children) === 0;
  return (
    <TabListContext.Provider value={{ orientation, overflow }}>
      <TabsPrimitive.Root
        value={value}
        onValueChange={onValueChange}
        orientation={orientation}
        className="contents"
      >
        <TabsPrimitive.List
          ref={ref}
          data-slot="tablist"
          loop={loop}
          aria-label={label}
          role={empty ? undefined : "tablist"}
          aria-orientation={empty ? undefined : orientation}
          className={cn(tabListVariants({ orientation, overflow }), className)}
          {...props}
        >
          {children}
        </TabsPrimitive.List>
      </TabsPrimitive.Root>
    </TabListContext.Provider>
  );
});

export const TabListItem = React.forwardRef<HTMLButtonElement, TabListItemProps>(function TabListItem(
  { className, children, ...props },
  ref,
) {
  const { orientation, overflow } = React.useContext(TabListContext);
  return (
    <TabsPrimitive.Trigger
      ref={ref}
      data-slot="tab"
      className={cn(tabListItemVariants({ orientation, overflow }), className)}
      {...props}
    >
      <span className="min-w-0 truncate">{children}</span>
    </TabsPrimitive.Trigger>
  );
});

export { tabListVariants, tabListItemVariants };
