import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";

/** Tone is the fill, border the edge, radius the corner. The container holds children only. */
const cardVariants = cva("group/card relative min-w-0 overflow-hidden text-content-1", {
  variants: {
    tone: {
      base: "bg-transparent",
      "surface-2": "bg-surface-2",
      "surface-3": "bg-surface-3",
    },
    border: {
      none: "border-0",
      solid: "border border-edge-2",
      dashed: "border border-dashed border-edge-2",
    },
    radius: {
      panel: "rounded-(--radius-panel)",
      control: "rounded-(--radius-control)",
    },
  },
  defaultVariants: { tone: "surface-2", border: "solid", radius: "panel" },
});

const cardHeadingVariants = cva("block min-w-0 truncate text-[0.857rem] leading-[1.2rem]", {
  variants: {
    context: {
      page: "font-semibold text-content-2",
      search: "font-medium text-content-2",
    },
  },
  defaultVariants: { context: "page" },
});

export interface CardProps extends React.HTMLAttributes<HTMLDivElement>, VariantProps<typeof cardVariants> {
  /** page | search: search shows the breadcrumb as the heading. */
  context?: "page" | "search";
  /** Hairline between child rows (edge-1). */
  divided?: boolean;
  /** Render onto the single child; the header and rows move inside it. */
  asChild?: boolean;
  /** Group heading; labels the group when set. */
  label?: string;
  /** Search context: the path shown as the heading. */
  breadcrumb?: string;
  /** Secondary line under the heading. */
  hint?: string;
  /** A change is pending; shows the dot and discard when onDiscard is set. */
  changed?: boolean;
  /** The card holds the note editor; focus within reveals its toolbar. */
  note?: boolean;
  /** Discard action handler. Its presence makes the card discardable. */
  onDiscard?: () => void;
}

function Rows({ divided, children }: { divided: boolean; children?: React.ReactNode }) {
  if (children === undefined || children === null || children === false) return null;
  return <div className={cn(divided && "divide-y divide-edge-1")}>{children}</div>;
}

function NoteBody({ label }: { label: string }) {
  return (
    <>
      <div
        data-slot="editor"
        role="textbox"
        aria-multiline="true"
        aria-label={label || "Note"}
        tabIndex={0}
        contentEditable
        className="min-h-[4.5rem] px-3 py-2.5 text-[0.929rem] leading-[1.357rem] text-content-1 outline-none focus-visible:ring-2 focus-visible:ring-focus-1"
      />
      <div
        data-slot="editorToolbar"
        role="toolbar"
        aria-label="Formatting"
        className="hidden items-center gap-0.5 border-t border-edge-1 px-2 py-1 group-focus-within/card:flex"
      >
        <Button size="sm" shape="icon" variant="ghost" icon="format_bold" ariaLabel="Bold" description="Bold" />
        <Button size="sm" shape="icon" variant="ghost" icon="format_italic" ariaLabel="Italic" description="Italic" />
        <Button size="sm" shape="icon" variant="ghost" icon="format_list_bulleted" ariaLabel="Bulleted list" description="Bulleted list" />
        <Button size="sm" shape="icon" variant="ghost" icon="link" ariaLabel="Link" description="Link" />
      </div>
    </>
  );
}

export const Card = React.forwardRef<HTMLDivElement, CardProps>(function Card(
  {
    tone, border, radius, context, divided = false, asChild = false, label = "", breadcrumb = "", hint = "",
    changed = false, note = false, onDiscard, className, children, ...props
  },
  ref,
) {
  const headingId = React.useId();
  const headingText = breadcrumb || label;
  const pending = changed && onDiscard !== undefined;
  const surface = cn(cardVariants({ tone, border, radius }), className);
  const group = {
    role: label !== "" ? ("group" as const) : undefined,
    "aria-labelledby": label !== "" && headingText !== "" ? headingId : undefined,
  };

  // changeDot and discard are fixed parts: always mounted, hidden unless a change is pending.
  const showHeader = headingText !== "" || hint !== "" || pending;
  const header = (
    <div className={cn("flex items-start gap-2 px-3 pt-2.5 pb-1.5", !showHeader && "hidden")}>
      <div className="min-w-0 flex-1">
        {headingText !== "" && (
          <div data-slot="heading" role="heading" aria-level={3} id={headingId} className={cardHeadingVariants({ context })}>
            {headingText}
          </div>
        )}
        {hint !== "" && (
          <div data-slot="hint" className="text-[0.786rem] leading-[1.1rem] text-content-3">
            {hint}
          </div>
        )}
      </div>
      <div className="flex shrink-0 items-center gap-1.5">
        <span
          data-slot="changeDot"
          aria-hidden="true"
          className={cn("size-[0.429rem] rounded-full bg-content-3", !pending && "hidden")}
        />
        <Button
          data-slot="discard"
          variant="ghost"
          size="sm"
          tone="default"
          label="Discard"
          onClick={onDiscard}
          className={cn(!pending && "hidden")}
        />
      </div>
    </div>
  );

  if (asChild) {
    const child = React.Children.only(children) as React.ReactElement<{ children?: React.ReactNode }>;
    return (
      <Slot ref={ref} data-slot="container" {...group} {...props} className={surface}>
        {React.cloneElement(child, undefined, header, <Rows divided={divided}>{child.props.children}</Rows>)}
      </Slot>
    );
  }

  return (
    <div ref={ref} data-slot="container" {...group} {...props} className={surface}>
      {header}
      <Rows divided={divided}>{note ? <NoteBody label={label} /> : children}</Rows>
    </div>
  );
});

export { cardVariants };
