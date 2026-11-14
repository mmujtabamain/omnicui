import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { Icon } from "./icon";
import { Switch } from "./switch";
import { Text } from "./text";

/** The row frame. A search-only row is shown alone, so it gets its own panel surface. */
const settingRowVariants = cva("flex min-w-0 w-full flex-col gap-[0.214rem]", {
  variants: {
    searchOnly: {
      true: "rounded-(--radius-panel) border border-edge-1 bg-surface-2 px-[0.714rem] py-[0.429rem]",
      false: "",
    },
  },
  defaultVariants: { searchOnly: false },
});

/** Title line: inline puts the control on the right, stacked puts it under the text. */
const settingRowBodyVariants = cva("flex min-w-0 w-full", {
  variants: {
    layout: {
      inline: "flex-row items-center justify-between gap-[0.857rem]",
      stacked: "flex-col items-stretch gap-[0.5rem]",
    },
    // One min-height per row: shortcut rows are 36px, every other row is the sm 28px line.
    shortcut: {
      true: "min-h-[2.571rem]",
      false: "min-h-[2rem]",
    },
  },
  defaultVariants: { layout: "inline", shortcut: false },
});

/** Segmented control: a 28px tray of radio segments. */
const segmentedVariants = cva("h-[2rem] items-stretch gap-[0.143rem] rounded-(--radius-field) bg-wash-2 p-[0.143rem]", {
  variants: {
    layout: {
      inline: "inline-flex w-auto shrink-0",
      stacked: "flex w-full",
    },
  },
  defaultVariants: { layout: "inline" },
});

/** A segment is a radio: its checked state is aria-checked, shown as the raised tray chip. */
const segmentVariants = cva(
  [
    "h-full min-w-0 rounded-(--radius-field) px-[0.714rem] text-[0.857rem] font-medium",
    "aria-checked:bg-surface-3 aria-checked:text-content-1 aria-checked:shadow-lift",
    "transition-[background-color,color,box-shadow] duration-(--duration-fast) ease-standard",
  ],
  {
    variants: {
      layout: {
        inline: "shrink-0",
        stacked: "flex-1",
      },
    },
    defaultVariants: { layout: "inline" },
  },
);

const SEGMENT_SLOTS = ["segmentOne", "segmentTwo", "segmentThree"] as const;
const ARROW_STEP: Record<string, number> = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 };
const CONTROL_TEXT = "text-[0.857rem]";

export interface SettingRowProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "title"> {
  /** Setting name; names the group and the control. */
  title?: string;
  /** Supporting text. "" = none. */
  description?: string;
  /** Note, or the rebind error line. "" = none. */
  footnote?: string;
  footnoteTone?: "note" | "error";
  layout?: "inline" | "stacked";
  size?: "sm";
  control?: "none" | "toggle" | "segmented" | "pathButton" | "button" | "hotkey";
  /** Toggle state. */
  checked?: boolean;
  /** Segment labels, pipe-separated. */
  choices?: string;
  /** Selected segment label; for pathButton, the path shown. "" = none. */
  choice?: string;
  /** Leading predicate icon. */
  icon?: boolean;
  /** Read-only trailing text. "" = none. */
  value?: string;
  /** The hotkey keycap is recording. */
  recording?: boolean;
  /** The row is the only search result. */
  searchOnly?: boolean;
  /** Title tone: primary for a row with a control, secondary for a read-only row. */
  tone?: "primary" | "secondary";
  /** The chord shown on a hotkey keycap. */
  keys?: string;
  onCheckedChange?: (checked: boolean) => void;
  onChoiceChange?: (value: string) => void;
  onRecordingChange?: (recording: boolean) => void;
  onPathPress?: () => void;
  onButtonPress?: () => void;
}

export const SettingRow = React.forwardRef<HTMLDivElement, SettingRowProps>(function SettingRow(
  {
    title = "",
    description = "",
    footnote = "",
    footnoteTone = "note",
    layout = "inline",
    size = "sm",
    control = "none",
    checked = false,
    choices = "System|Light|Dark",
    choice = "",
    icon = false,
    value = "",
    recording = false,
    searchOnly = false,
    tone = "primary",
    keys = "",
    onCheckedChange,
    onChoiceChange,
    onRecordingChange,
    onPathPress,
    onButtonPress,
    className,
    ...props
  },
  ref,
) {
  const titleId = React.useId();
  const footnoteId = React.useId();
  const hasControl = control !== "none";
  const showFootnote = footnote !== "";
  const isError = footnoteTone === "error";
  const describedBy = showFootnote ? footnoteId : undefined;
  const segments = choices.split("|").slice(0, SEGMENT_SLOTS.length);
  // Buttons fill the full line when stacked; inline they keep their own width.
  const buttonWidth = layout === "stacked" ? "full" : "content";

  // WAI-ARIA radio group: one tab stop (the checked segment), arrow keys move focus and check.
  function moveFocus(event: React.KeyboardEvent<HTMLButtonElement>, index: number) {
    const step = ARROW_STEP[event.key];
    if (step === undefined) return;
    event.preventDefault();
    const next = (index + step + segments.length) % segments.length;
    const label = segments[next];
    const radios = event.currentTarget.parentElement?.querySelectorAll<HTMLElement>('[role="radio"]');
    radios?.[next]?.focus();
    if (label !== undefined) onChoiceChange?.(label);
  }

  function renderControl(): React.ReactNode {
    if (control === "toggle")
      return (
        <Switch
          data-slot="toggle"
          ariaLabel={title}
          aria-describedby={describedBy}
          checked={checked}
          onCheckedChange={onCheckedChange}
        />
      );
    if (control === "segmented") {
      const selectedIndex = segments.indexOf(choice);
      return (
        <div
          role="radiogroup"
          aria-label={title}
          aria-describedby={describedBy}
          data-slot="control"
          className={segmentedVariants({ layout })}
        >
          {segments.map((label, index) => (
            <Button
              key={label}
              data-slot={SEGMENT_SLOTS[index]}
              role="radio"
              aria-checked={label === choice}
              tabbable={selectedIndex === -1 ? index === 0 : label === choice}
              variant="ghost"
              size="lg"
              width={layout === "stacked" ? "full" : "content"}
              label={label}
              className={segmentVariants({ layout })}
              onClick={() => onChoiceChange?.(label)}
              onKeyDown={(event) => moveFocus(event, index)}
            />
          ))}
        </div>
      );
    }
    if (control === "pathButton")
      return (
        <Button
          data-slot="button"
          ariaLabel={title}
          aria-describedby={describedBy}
          variant="outline"
          size="lg"
          width={buttonWidth}
          block
          icon="folder"
          label={choice}
          className={CONTROL_TEXT}
          onClick={onPathPress}
        />
      );
    if (control === "button")
      return (
        <Button
          data-slot="button"
          ariaLabel={title}
          aria-describedby={describedBy}
          variant="neutral"
          size="lg"
          width={buttonWidth}
          label={title}
          className={CONTROL_TEXT}
          onClick={onButtonPress}
        />
      );
    if (control === "hotkey")
      return (
        <Button
          data-slot="button"
          ariaLabel={title}
          aria-describedby={describedBy}
          variant="outline"
          size="lg"
          width="content"
          mono
          label={recording ? "Press keys" : keys}
          className={cn("min-w-[2.5rem]", recording && "border-accent-500 text-accent-600")}
          onClick={() => onRecordingChange?.(true)}
          onKeyDown={(event) => {
            if (recording && event.key === "Escape") {
              event.preventDefault();
              onRecordingChange?.(false);
            }
          }}
        />
      );
    return null;
  }

  const controlNode = renderControl();

  const valueNode =
    value !== "" ? (
      <Text data-slot="value" size="md" tone="secondary" numeric="tabular" className="shrink-0">
        {value}
      </Text>
    ) : null;

  const textRow = (
    <div className="flex min-w-0 flex-1 items-center gap-[0.5rem]">
      {icon && (
        <Icon data-part="ui.settingrow.icon" data-slot="icon" name="filter_list" size={15} weight={300} tone="inherit" className="shrink-0 text-content-2" />
      )}
      <div className="flex min-w-0 flex-1 flex-col gap-[0.143rem]">
        <Text id={titleId} data-slot="title" size="md" weight="medium" tone={tone} className="leading-[1.393rem]">
          {title}
        </Text>
        {description !== "" && (
          <Text as="p" data-slot="description" size="sm" tone="secondary" className="leading-[1.2rem]">
            {description}
          </Text>
        )}
      </div>
    </div>
  );

  const body = (
    <div className={settingRowBodyVariants({ layout, shortcut: control === "hotkey" })}>
      {textRow}
      {layout === "stacked" ? (
        <>
          {controlNode}
          {valueNode}
        </>
      ) : (
        (controlNode !== null || valueNode !== null) && (
          <div className="flex shrink-0 items-center gap-[0.5rem]">
            {controlNode}
            {valueNode}
          </div>
        )
      )}
    </div>
  );

  const footnoteNode = showFootnote ? (
    <Text
      as="p"
      id={footnoteId}
      data-slot="footnote"
      size="xs"
      tone={isError ? "danger" : "secondary"}
      className="leading-[1.1rem]"
    >
      {isError ? (
        <Text as="span" data-slot="rebindError" role="status" live="polite" size="xs" tone="danger">
          {footnote}
        </Text>
      ) : (
        footnote
      )}
    </Text>
  ) : null;

  return (
    <div
      ref={ref}
      data-slot="row"
      data-size={size}
      role={hasControl ? "group" : undefined}
      aria-labelledby={hasControl ? titleId : undefined}
      className={cn(settingRowVariants({ searchOnly }), className)}
      {...props}
    >
      {body}
      {footnoteNode}
    </div>
  );
});

export { settingRowVariants, settingRowBodyVariants, segmentedVariants, segmentVariants };
