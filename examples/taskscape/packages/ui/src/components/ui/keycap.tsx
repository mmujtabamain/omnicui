import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { HotkeyHint } from "./hotkey-hint";

/**
 * A shortcut keycap (shadcn Button styling on a native button, no Radix primitive). Idle shows the
 * binding as glyphs (HotkeyHint sm) or the unbound label; recording shows the prompt in a fixed
 * width box with the neutral focus ring. Sizes at 1rem = 14px: height 1.5rem, idle minimum width
 * 3.5rem, recording width 7.214rem, label text 0.786rem on a 1.179rem line, radius-field corners.
 */
export const keycapVariants = cva(
  [
    "inline-flex min-w-0 items-center justify-center whitespace-nowrap select-none font-medium",
    "h-[1.5rem] px-1.5 text-[0.786rem] leading-[1.179rem] rounded-(--radius-field)",
    "outline-none transition-[color,background-color,box-shadow] duration-(--duration-fast) ease-standard",
    "focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-offset-0",
    "disabled:pointer-events-none disabled:opacity-45",
  ],
  {
    variants: {
      mode: {
        idle: "min-w-[3.5rem] bg-surface-0 text-content-2 hover:bg-wash-2 hover:text-content-1",
        recording: "w-[7.214rem] bg-surface-0 text-content-1 ring-1 ring-inset ring-focus-1",
      },
    },
    defaultVariants: { mode: "idle" },
  },
);

type Variants = VariantProps<typeof keycapVariants>;

const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "AltGraph", "Meta", "OS"]);

export interface KeycapProps extends React.HTMLAttributes<HTMLDivElement> {
  /** idle shows the binding; recording waits for the next combination. */
  mode?: Variants["mode"];
  /** unbound wins when accel is empty. custom shows the reset control beside the keycap. */
  binding?: "default" | "custom" | "unbound";
  /** Stored accelerator, for example "Shift+F5". Empty or null means unbound. */
  accel?: string | null;
  /** Name of the command the row binds, used in the accessible name. */
  actionName?: string;
  /** Shown and announced while recording. */
  prompt?: string;
  /** Muted text shown when unbound. */
  unboundLabel?: string;
  /** Comma-separated combos the platform owns; a recorded one is refused, not captured. */
  reserved?: string;
  /** app works while the window is focused (the only scope on web). global is desktop only. */
  scope?: "app" | "global";
  disabled?: boolean;
  onStartRecording?: () => void;
  /** Called with the recorded combo, or null when a bare Backspace or Delete unbinds the row. */
  onCapture?: (accel: string | null) => void;
  onCancel?: () => void;
  onReset?: () => void;
  /** Called with a reserved combo that was pressed while recording. */
  onRefused?: (accel: string) => void;
}

/** Accelerator notation: Control, Alt, Shift, Meta, then the key, joined with "+". */
function acceleratorOf(e: React.KeyboardEvent): string {
  const mods: string[] = [];
  if (e.ctrlKey) mods.push("Control");
  if (e.altKey) mods.push("Alt");
  if (e.shiftKey) mods.push("Shift");
  if (e.metaKey) mods.push("Meta");
  const key = e.key === " " ? "Space" : e.key.length === 1 ? e.key.toUpperCase() : e.key;
  return [...mods, key].join("+");
}

export const Keycap = React.forwardRef<HTMLDivElement, KeycapProps>(function Keycap(
  {
    mode = "idle",
    binding = "default",
    accel = "",
    actionName = "",
    prompt = "Type shortcut…",
    unboundLabel = "None",
    reserved = "",
    scope = "app",
    disabled = false,
    onStartRecording,
    onCapture,
    onCancel,
    onReset,
    onRefused,
    className,
    ...props
  },
  ref,
) {
  const descriptionId = React.useId();
  const [refusal, setRefusal] = React.useState<string | null>(null);
  const recording = mode === "recording";
  const combo = accel ?? "";
  const unbound = binding === "unbound" || combo === "";
  const reservedCombos = reserved.split(",").map((c) => c.trim()).filter(Boolean);
  const spoken = combo.split("+").join(" ");
  const label = [actionName, recording ? prompt : unbound ? unboundLabel : spoken].filter(Boolean).join(", ");
  const showReset = binding === "custom" && !unbound && !recording;

  const onKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>) => {
    if (!recording || disabled) return;
    // Tab passes through; the blur that follows cancels the recording.
    if (e.key === "Tab") return;
    if (e.key === "Escape") {
      e.preventDefault();
      setRefusal(null);
      onCancel?.();
      return;
    }
    const bare = !(e.ctrlKey || e.altKey || e.shiftKey || e.metaKey);
    // A bare Enter or Space cannot be bound: the click that follows cancels.
    if (bare && (e.key === "Enter" || e.key === " ")) return;
    e.preventDefault();
    if (MODIFIER_KEYS.has(e.key)) return;
    if (bare && (e.key === "Backspace" || e.key === "Delete")) {
      setRefusal(null);
      onCapture?.(null);
      return;
    }
    const next = acceleratorOf(e);
    if (reservedCombos.includes(next)) {
      setRefusal(`${next} is reserved`);
      onRefused?.(next);
      return;
    }
    setRefusal(null);
    onCapture?.(next);
  };

  return (
    <div
      ref={ref}
      role="group"
      data-slot="group"
      data-scope={scope}
      className={cn("inline-flex items-center gap-1", className)}
      {...props}
    >
      <button
        type="button"
        data-slot="keycap"
        disabled={disabled}
        aria-label={label}
        aria-keyshortcuts={!recording && !unbound ? combo : undefined}
        aria-describedby={recording ? undefined : descriptionId}
        onKeyDown={onKeyDown}
        onBlur={() => {
          if (recording) onCancel?.();
        }}
        onClick={() => {
          setRefusal(null);
          if (recording) onCancel?.();
          else onStartRecording?.();
        }}
        className={keycapVariants({ mode })}
      >
        {recording ? prompt : unbound ? unboundLabel : <HotkeyHint accel={combo} size="sm" tone="inherit" />}
      </button>
      {!recording && (
        <span id={descriptionId} className="sr-only">
          Press to change
        </span>
      )}
      {showReset && (
        <Button
          data-slot="reset"
          shape="icon"
          size="sm"
          icon="restart_alt"
          ariaLabel={`Reset ${actionName}`.trim()}
          disabled={disabled}
          onClick={() => onReset?.()}
        />
      )}
      <span data-slot="announcement" role="status" aria-live="polite" className="sr-only">
        {recording ? (refusal ?? prompt) : ""}
      </span>
    </div>
  );
});
