import { AttachmentPreview } from "../components/ui/attachment-preview";

/** The stage gives the preview a definite height, as the lightbox does; the widget fills it. */
function AttachmentPreviewHarness({ emit, kind = "image", ...props }: any) {
  return (
    <div className="flex h-dvh w-full min-w-0 p-3">
      <AttachmentPreview
        {...props}
        kind={kind}
        onLoaded={() => emit("loaded", { kind })}
        onPlaying={() => emit("playing", { kind })}
        onError={() => emit("error", { kind })}
      />
    </div>
  );
}

export default { id: "ui.attachmentpreview", Component: AttachmentPreviewHarness, events: ["loaded", "playing", "error"] };
