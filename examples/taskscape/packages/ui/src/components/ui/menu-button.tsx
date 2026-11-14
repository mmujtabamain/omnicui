import * as React from "react";
import * as DropdownMenuPrimitive from "@radix-ui/react-dropdown-menu";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { Icon } from "./icon";
import { menuPanelVariants } from "./menu";

/**
 * The trigger look, on top of Button. Sizes at 1rem = 14px: sm is 21px high with a 12px/18px label; lg is 28px
 * with a 12px/18px label (bulk move); default is 28px with a 14px/21px label (the pill size); icon is a 28px square.
 * The pill (title bar project switcher) is 85px at least, with a 14px/21px label at weight 600.
 * The open look is the hover wash (spec open item 3); Radix sets data-state="open" on the trigger.
 */
export const menuButtonVariants = cva("font-medium data-[state=open]:bg-wash-2", {
  variants: {
    variant: {
      neutral: "bg-surface-0 text-content-1 hover:bg-wash-2",
      ghost: "bg-transparent text-content-2 hover:bg-wash-1 hover:text-content-1",
      pill: "bg-surface-0 font-semibold text-content-1 hover:bg-wash-2",
    },
    size: {
      sm: "text-[0.857rem] leading-[1.286rem]",
      lg: "text-[0.857rem] leading-[1.286rem]",
      default: "text-[1rem] leading-[1.5rem]",
      icon: "",
    },
  },
  compoundVariants: [
    { variant: "pill", size: ["lg", "default"], className: "min-w-[6.072rem] text-[1rem] leading-[1.5rem]" },
  ],
  defaultVariants: { variant: "ghost", size: "sm" },
});

type MenuButtonVariants = VariantProps<typeof menuButtonVariants>;

export interface MenuButtonProps extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "type" | "children"> {
  variant?: MenuButtonVariants["variant"];
  size?: MenuButtonVariants["size"];
  /** Leading glyph name; empty for none. */
  leadingIcon?: string;
  /** Trailing chevron name; empty for none. */
  trailingIcon?: string;
  /** Current value; empty shows the placeholder. */
  label?: string;
  /** Shown while label is empty. */
  placeholder?: string;
  disabled?: boolean;
  /** Accessible name; empty means the visible text is the name. */
  ariaLabel?: string;
  /** Controlled open state. */
  open?: boolean;
  /** Fired with the requested state on a press, Enter, Space, ArrowDown or Escape. */
  onOpenChange?: (open: boolean) => void;
  /** The menu rows (MenuItem), shown in the panel while open. */
  menu?: React.ReactNode;
}

/**
 * A button that opens a menu of actions or choices. Radix DropdownMenu supplies the trigger behaviour
 * (press, Enter, Space, ArrowDown, Escape, focus return); Button supplies the look.
 */
export const MenuButton = React.forwardRef<HTMLButtonElement, MenuButtonProps>(function MenuButton(
  {
    variant = "ghost",
    size = "sm",
    leadingIcon = "",
    trailingIcon = "",
    label = "",
    placeholder = "",
    disabled = false,
    ariaLabel = "",
    open = false,
    onOpenChange,
    menu,
    className,
    ...props
  },
  ref,
) {
  // Radix can report one press twice (a dismiss and a toggle) in the same event. Report it once.
  const last = React.useRef<boolean | null>(null);
  const change = (next: boolean) => {
    if (next === open || next === last.current) return;
    last.current = next;
    window.setTimeout(() => {
      last.current = null;
    }, 0);
    onOpenChange?.(next);
  };

  const iconOnly = size === "icon";
  const shown = label !== "" ? label : placeholder;
  const glyphSize = size === "sm" ? 14 : 16;
  // Button gives the height and radius: sm 21px; lg 28px; the pill and default sizes use xl (28px, 7px radius).
  const buttonSize = size === "sm" ? "sm" : variant === "pill" || size === "default" ? "xl" : "lg";

  return (
    <DropdownMenuPrimitive.Root open={open} onOpenChange={change} modal={false}>
      <DropdownMenuPrimitive.Trigger asChild disabled={disabled}>
        <Button
          asChild
          ref={ref}
          variant={variant === "pill" ? "neutral" : variant}
          size={buttonSize}
          shape={iconOnly ? "icon" : "text"}
          width="content"
          className={cn(menuButtonVariants({ variant, size }), className)}
        >
          <button
            type="button"
            disabled={disabled}
            aria-label={ariaLabel || undefined}
            {...props}
            data-slot="trigger"
          >
            {leadingIcon !== "" && (
              <Icon data-slot="leadingIcon" name={leadingIcon} size={glyphSize} weight={300} tone="inherit" className="shrink-0" />
            )}
            {!iconOnly && shown !== "" && (
              <span data-slot="label" className={cn("min-w-0 truncate", label === "" && "text-content-3")}>
                {shown}
              </span>
            )}
            {trailingIcon !== "" && (
              <Icon data-slot="trailingIcon" name={trailingIcon} size={15} weight={300} tone="inherit" className="shrink-0 text-content-3" />
            )}
          </button>
        </Button>
      </DropdownMenuPrimitive.Trigger>
      <DropdownMenuPrimitive.Portal>
        <DropdownMenuPrimitive.Content
          data-slot="menu"
          side="bottom"
          align="start"
          sideOffset={4}
          collisionPadding={4}
          className={menuPanelVariants({ width: variant === "pill" ? "project" : "content" })}
        >
          {menu}
        </DropdownMenuPrimitive.Content>
      </DropdownMenuPrimitive.Portal>
    </DropdownMenuPrimitive.Root>
  );
});
