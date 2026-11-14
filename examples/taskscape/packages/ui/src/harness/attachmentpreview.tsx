import { AttachmentPreview } from "../components/ui/attachment-preview";

function AttachmentPreviewHarness({ emit, kind = "image", ...props }: any) {
  return (
    <AttachmentPreview
      {...props}
      kind={kind}
      onLoaded={() => emit("loaded", { kind })}
      onPlaying={() => emit("playing", { kind })}
      onError={() => emit("error", { kind })}
    />
  );
}

export default { id: "ui.attachmentpreview", Component: AttachmentPreviewHarness, events: ["loaded", "playing", "error"] };
