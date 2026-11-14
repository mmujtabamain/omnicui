import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";

/** A 94px square tile (6.714rem) with a 7px corner (control) and a 1px edge; hover takes edge-3. */
export const attachmentTileVariants = cva(
  [
    "group/attachment-tile relative flex size-[6.714rem] shrink-0 select-none items-center justify-center overflow-hidden",
    "rounded-(--radius-control) border border-edge-2 bg-surface-3 p-0 outline-none",
    "transition-[border-color] duration-(--duration-fast) ease-standard",
    "hover:border-edge-3 focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-offset-0",
  ],
  {
    variants: {
      kind: {
        image: "",
        link: "text-content-3",
      },
    },
    defaultVariants: { kind: "image" },
  },
);

export type AttachmentKind = "image" | "link";

export interface AttachmentTileProps extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "type"> {
  /** Content type: image shows a thumbnail, link shows a glyph. */
  kind?: AttachmentKind;
  /** Attachment name; the accessible name. */
  name?: string;
  /** Image kind only. */
  thumbnailSrc?: string;
  /** Carries the attachment MIME type when dragged. */
  draggable?: boolean;
  /** Click, Enter or Space: the lightbox (image) or the link (link). */
  onOpen?: () => void;
  /** Right-click, the context-menu key or Shift+F10: the caller shows its menu. */
  onMenuOpen?: () => void;
}

export const AttachmentTile = React.forwardRef<HTMLButtonElement, AttachmentTileProps>(function AttachmentTile(
  {
    kind = "image",
    name = "",
    thumbnailSrc = "",
    draggable = true,
    onOpen,
    onMenuOpen,
    onClick,
    onContextMenu,
    onKeyDown,
    onDragStart,
    className,
    ...props
  },
  ref,
) {
  const link = kind === "link";
  return (
    <button
      ref={ref}
      type="button"
      data-slot="tile"
      data-kind={kind}
      aria-label={name || undefined}
      draggable={draggable}
      className={cn(attachmentTileVariants({ kind }), className)}
      onClick={(e) => {
        onClick?.(e);
        onOpen?.();
      }}
      onContextMenu={(e) => {
        onContextMenu?.(e);
        e.preventDefault();
        onMenuOpen?.();
      }}
      onKeyDown={(e) => {
        onKeyDown?.(e);
        if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) {
          e.preventDefault();
          onMenuOpen?.();
        }
      }}
      onDragStart={(e) => {
        e.dataTransfer.effectAllowed = "copy";
        onDragStart?.(e);
      }}
      {...props}
    >
      {link ? (
        <Icon data-slot="glyph" name="link" size={22} tone="inherit" />
      ) : (
        <img
          data-slot="thumb"
          src={thumbnailSrc}
          alt=""
          draggable={false}
          className="absolute inset-0 size-full object-cover"
        />
      )}
      <span
        data-slot="overlay"
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 hidden items-center justify-center group-hover/attachment-tile:flex"
      >
        {/* The glyph is drawn by a pseudo-element, so the tile's rendered text never includes the overlay name. */}
        <span className="material-symbols-outlined flex size-[1.714rem] items-center justify-center rounded-full bg-surface-2 text-content-1 shadow-lift [font-family:'Material_Symbols_Outlined'] [font-variation-settings:'wght'_300] before:content-['open\_in\_full'] before:whitespace-nowrap before:text-[1.143rem] before:leading-none" />
      </span>
    </button>
  );
});
