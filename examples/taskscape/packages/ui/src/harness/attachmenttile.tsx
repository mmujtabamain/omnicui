import { AttachmentTile } from "../components/ui/attachment-tile";

function AttachmentTileHarness({ emit, kind = "image", ...props }: any) {
  return (
    <AttachmentTile
      {...props}
      kind={kind}
      onOpen={() => emit("opened", { kind })}
      onMenuOpen={() => emit("contextMenuOpened", { kind })}
      onDragStart={() => emit("dragStarted", { kind })}
    />
  );
}

export default {
  id: "ui.attachmenttile",
  Component: AttachmentTileHarness,
  events: ["opened", "contextMenuOpened", "dragStarted"],
};
