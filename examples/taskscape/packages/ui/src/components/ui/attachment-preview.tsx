import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";

/** One element per kind: image img, video video, audio panel, pdf embed, text pre. */
const attachmentPreviewVariants = cva("", {
  variants: {
    kind: {
      image: "max-h-full max-w-full rounded-(--radius-control) object-contain shadow-menu",
      video: "max-h-full max-w-full rounded-(--radius-control) shadow-menu",
      audio: "w-full max-w-md flex-col items-center gap-3 rounded-(--radius-panel) bg-surface-2 px-8 py-10",
      pdf: "h-full w-full max-w-4xl rounded-(--radius-control)",
      text: "max-h-full w-full max-w-3xl overflow-auto rounded-(--radius-control) bg-surface-2 p-5 text-[0.893rem] leading-5 text-content-1 select-text",
    },
  },
  defaultVariants: { kind: "image" },
});

type Variants = VariantProps<typeof attachmentPreviewVariants>;
export type AttachmentKind = NonNullable<Variants["kind"]>;

/** The text kind is capped at this many characters. */
const TEXT_LIMIT = 262144;

const FALLBACK_ICON: Record<AttachmentKind, string> = {
  image: "image_not_supported",
  video: "movie",
  audio: "audio_file",
  pdf: "picture_as_pdf",
  text: "description",
};

type Status = "loading" | "loaded" | "error";

export interface AttachmentPreviewProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "onError"> {
  kind?: AttachmentKind;
  /** Attachment URL (web) or path (desktop). */
  src?: string;
  /** Alt text for the image kind; accessible label for the others. */
  name?: string;
  /** Starts playback on mount for video and audio. */
  autoPlay?: boolean;
  /** Text content for the text kind. */
  text?: string;
  /** The attachment has loaded (text: at mount). */
  onLoaded?: () => void;
  /** Playback started (video and audio). */
  onPlaying?: () => void;
  /** The attachment failed; the EmptyState card replaces it. */
  onError?: () => void;
}

export const AttachmentPreview = React.forwardRef<HTMLDivElement, AttachmentPreviewProps>(function AttachmentPreview(
  { kind = "image", src = "", name = "", autoPlay = true, text = "", onLoaded, onPlaying, onError, className, ...props },
  ref,
) {
  // The outcome is tied to the kind and source it was reached for, so a new attachment starts loading again.
  const [outcome, setOutcome] = React.useState<{ key: string; status: Status }>({ key: "", status: "loading" });
  const key = `${kind}\n${src}`;
  const settle = (next: Status) => setOutcome({ key, status: next });
  const status: Status = kind === "text" ? "loaded" : outcome.key === key ? outcome.status : "loading";

  // Text is loaded at mount; announce it once per content.
  const announcedText = React.useRef<string | null>(null);
  React.useEffect(() => {
    if (kind !== "text") {
      announcedText.current = null;
      return;
    }
    if (announcedText.current === text) return;
    announcedText.current = text;
    onLoaded?.();
  }, [kind, text, onLoaded]);

  const load = () => {
    settle("loaded");
    onLoaded?.();
  };
  const fail = () => {
    settle("error");
    onError?.();
  };

  const failed = status === "error";
  const loading = kind === "image" && status === "loading";

  // loading and fallback are always mounted (the contract needs them to exist); visibility decides.
  return (
    <div
      ref={ref}
      className={cn("relative flex h-full w-full min-h-0 min-w-0 items-center justify-center", className)}
      {...props}
    >
      <div
        role="status"
        data-slot="fallback"
        className={cn(
          failed ? "flex" : "hidden",
          "w-full max-w-md flex-col items-center gap-2 rounded-(--radius-panel) bg-surface-2 px-8 py-10 text-center",
        )}
      >
        <Icon name={FALLBACK_ICON[kind]} size={40} weight={200} tone="inherit" className="text-content-3" />
        <span className="text-[0.929rem] font-medium text-content-1">Preview unavailable</span>
        {name !== "" && <span className="max-w-full truncate text-[0.857rem] text-content-3">{name}</span>}
      </div>

      <div className={cn(loading ? "flex" : "hidden", "items-center gap-1.5 text-content-3")}>
        <Icon name="progress_activity" size={16} tone="inherit" className="animate-spin" />
        <span data-slot="loading" role="status" className="text-[0.929rem]">
          Loading...
        </span>
      </div>

      {kind === "image" && (
        <img
          data-slot="image"
          src={src || undefined}
          alt={name}
          draggable={false}
          onLoad={load}
          onError={fail}
          className={cn(attachmentPreviewVariants({ kind: "image" }), (loading || failed) && "hidden")}
        />
      )}

      {kind === "video" && (
        <video
          data-slot="video"
          src={src || undefined}
          aria-label={name}
          controls
          autoPlay={autoPlay}
          playsInline
          onLoadedData={load}
          onPlaying={() => onPlaying?.()}
          onError={fail}
          className={cn(attachmentPreviewVariants({ kind: "video" }), failed && "hidden")}
        />
      )}

      {kind === "audio" && (
        <div
          data-slot="audio"
          className={cn(
            failed ? "hidden" : "flex flex-col items-center",
            attachmentPreviewVariants({ kind: "audio" }),
          )}
        >
          <Icon data-slot="glyph" name="audio_file" size={40} weight={200} tone="inherit" className="text-content-3" />
          <audio
            src={src || undefined}
            aria-label={name}
            controls
            autoPlay={autoPlay}
            onLoadedData={load}
            onPlaying={() => onPlaying?.()}
            onError={fail}
            className="w-full"
          />
        </div>
      )}

      {kind === "pdf" && (
        <embed
          data-slot="pdf"
          type="application/pdf"
          src={src || undefined}
          title={name}
          aria-label={name}
          onLoad={() => onLoaded?.()}
          className={attachmentPreviewVariants({ kind: "pdf" })}
        />
      )}

      {kind === "text" && (
        <pre data-slot="text" className={attachmentPreviewVariants({ kind: "text" })}>
          {text.slice(0, TEXT_LIMIT)}
        </pre>
      )}
    </div>
  );
});

export { attachmentPreviewVariants };
