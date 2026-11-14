// The web Attachment lightbox: a full-window overlay over the page that previews one attachment of
// a task at a large size, steps through the task's attachments, and offers open, rename and remove.
import * as React from "react";
import { AttachmentPreview } from "@taskscape/ui/attachment-preview";
import { Button } from "@taskscape/ui/button";
import { Footer } from "@taskscape/ui/footer";
import { Icon } from "@taskscape/ui/icon";
import { Input } from "@taskscape/ui/input";
import { Lightbox, LightboxMedia } from "@taskscape/ui/lightbox";
import { Overlay, OverlayTitle } from "@taskscape/ui/overlay";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { useTasks, useWrite } from "../../api/queries";
import type { Attachment, Id } from "../../api/types";
import { KIND_ICON, fileKindFor, splitName, type FileKind } from "./kinds";

/** What the screen reports to its host. Each callback is optional. */
export interface AttachmentLightboxEvents {
  closed?: (detail: { via: string; name: string }) => void;
  openedExternally?: (detail: { name: string }) => void;
  renamed?: (detail: { name: string }) => void;
  removed?: (detail: { name: string }) => void;
  stepped?: (detail: { name: string }) => void;
}

export interface AttachmentLightboxProps {
  /** The list the task is in; its tasks (with their attachments) are read from the API. */
  listId: Id;
  /** The task whose attachments the lightbox steps through. */
  taskId: Id;
  /** The attachment the lightbox opens on. */
  attachmentId: Id;
  events?: AttachmentLightboxEvents;
}

type Sheet = "rename" | "remove" | null;

/** Opens the attachment in the default app: a new browser tab on the web. */
function openAttachment(attachment: Attachment) {
  window.open(attachment.url, "_blank", "noopener");
}

export function AttachmentLightbox({ listId, taskId, attachmentId, events = {} }: AttachmentLightboxProps) {
  const tasks = useTasks({ list: listId });
  const rename = useWrite(ep.attachments.rename);
  const remove = useWrite(ep.attachments.remove);
  const [open, setOpen] = React.useState(true);
  const [currentId, setCurrentId] = React.useState<Id>(attachmentId);
  const [sheet, setSheet] = React.useState<Sheet>(null);

  const attachments: Attachment[] = tasks.data?.find((t) => t.id === taskId)?.attachments ?? [];
  const count = attachments.length;
  const index = attachments.findIndex((a) => a.id === currentId);
  const current = index >= 0 ? attachments[index] : undefined;

  const stepBy = (delta: number) => {
    if (count < 2 || index < 0) return;
    const next = attachments[(index + delta + count) % count];
    setCurrentId(next.id);
    events.stepped?.({ name: next.name });
  };

  // Arrow keys step while the lightbox is open with no sheet on top. Keys in a native video, audio
  // or text preview stay with that control.
  React.useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!open || sheet !== null || count < 2) return;
      const delta = event.key === "ArrowRight" ? 1 : event.key === "ArrowLeft" ? -1 : 0;
      if (delta === 0) return;
      const target = event.target;
      if (target instanceof Element && target.closest("video, audio, pre")) return;
      event.preventDefault();
      event.stopPropagation();
      stepBy(delta);
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  });

  if (!current) return null;

  const kind: FileKind = fileKindFor(current);
  const { base, tail } = splitName(current);
  const remote = current.kind === "link";
  const renameDetail = remote
    ? "Renames the label only — the linked file is untouched."
    : "The stored file is renamed to match.";

  const close = (via: "button" | "escape" | "stage") => {
    setOpen(false);
    events.closed?.({ via, name: current.name });
  };

  const submitRename = (value: string) => {
    const typed = value.trim();
    setSheet(null);
    if (typed === "" || typed === base) return;
    const full = typed + tail;
    if (full === current.name) return;
    rename.mutate([current.id, full], { onSuccess: () => events.renamed?.({ name: full }) });
  };

  const confirmRemove = () => {
    const target = current;
    const rest = attachments.filter((a) => a.id !== target.id);
    const next = rest.length > 0 ? rest[Math.min(index, rest.length - 1)] : undefined;
    setSheet(null);
    if (next) setCurrentId(next.id);
    else setOpen(false);
    remove.mutate([target.id], { onSuccess: () => events.removed?.({ name: target.name }) });
  };

  const openExternally = () => {
    openAttachment(current);
    events.openedExternally?.({ name: current.name });
  };

  const sheetCard = "w-[min(26rem,100%)] overflow-hidden";
  const sheetBody = "flex min-w-0 flex-col gap-[0.571rem] pt-[1.071rem] pr-[0.714rem] pb-[0.714rem] pl-[1.071rem]";

  return (
    <>
      <Lightbox open={open} onClose={close} label="Attachment">
        <div className="-mx-16 flex h-13 shrink-0 items-center gap-[0.429rem] px-[0.857rem]">
          <Icon data-part="app.attachmentlightbox.kindIcon" name={KIND_ICON[kind]} size="md" tone="onMediaMuted" className="shrink-0" />
          <Text
            as="span"
            data-part="app.attachmentlightbox.fileName"
            size="md"
            tone="onMedia"
            overflow="truncate"
            title={current.name}
            className="min-w-0 flex-1"
          >
            {current.name}
          </Text>
          {count > 1 && (
            <Text
              as="span"
              data-part="app.attachmentlightbox.counter"
              size="sm"
              tone="onMediaMuted"
              numeric="tabular"
              className="mr-1 shrink-0"
            >
              {`${index + 1} / ${count}`}
            </Text>
          )}
          <div className="flex shrink-0 items-center gap-[0.357rem]">
            <Button
              data-part="app.attachmentlightbox.openExternally"
              surface="media"
              variant="ghost"
              shape="icon"
              size="xl"
              icon="open_in_new"
              ariaLabel="Open externally"
              description="Open externally"
              onClick={openExternally}
            />
            <Button
              data-part="app.attachmentlightbox.rename"
              surface="media"
              variant="ghost"
              shape="icon"
              size="xl"
              icon="drive_file_rename_outline"
              ariaLabel="Rename"
              description="Rename"
              onClick={() => setSheet("rename")}
            />
            <Button
              data-part="app.attachmentlightbox.remove"
              surface="media"
              variant="ghost"
              tone="danger"
              shape="icon"
              size="xl"
              icon="delete"
              ariaLabel="Remove attachment"
              description="Remove attachment"
              className="text-danger-500"
              onClick={() => setSheet("remove")}
            />
            <Button
              data-part="app.attachmentlightbox.close"
              surface="media"
              variant="ghost"
              shape="icon"
              size="xl"
              icon="close"
              ariaLabel="Close"
              description="Close  Esc"
              onClick={() => close("button")}
            />
          </div>
        </div>

        {/* Clicks pass through this layer to the stage (which closes); the media and chevrons take their own. */}
        <div className="pointer-events-none relative -mx-16 flex min-h-0 flex-1 items-center justify-center">
          {count > 1 && (
            <Button
              data-part="app.attachmentlightbox.previous"
              surface="media"
              variant="filled"
              shape="round"
              size="step"
              icon="chevron_left"
              iconWeight={300}
              ariaLabel="Previous"
              description="Previous"
              className="pointer-events-auto absolute top-1/2 left-4 -translate-y-1/2"
              onClick={() => stepBy(-1)}
            />
          )}
          {count > 1 && (
            <Button
              data-part="app.attachmentlightbox.next"
              surface="media"
              variant="filled"
              shape="round"
              size="step"
              icon="chevron_right"
              iconWeight={300}
              ariaLabel="Next"
              description="Next"
              className="pointer-events-auto absolute top-1/2 right-4 -translate-y-1/2"
              onClick={() => stepBy(1)}
            />
          )}
          <LightboxMedia className="pointer-events-auto flex-none">
            <Body item={current} kind={kind} onOpen={openExternally} />
          </LightboxMedia>
        </div>
      </Lightbox>

      <Overlay
        open={sheet === "rename"}
        onOpenChange={(next) => {
          if (!next) setSheet(null);
        }}
        className={sheetCard}
      >
        <RenameForm
          base={base}
          tail={tail}
          detail={renameDetail}
          onSubmit={submitRename}
          onCancel={() => setSheet(null)}
        />
      </Overlay>

      <Overlay
        open={sheet === "remove"}
        role="alertdialog"
        onOpenChange={(next) => {
          if (!next) setSheet(null);
        }}
        className={sheetCard}
      >
        <div className={sheetBody}>
          <div className="flex min-w-0 items-start gap-[0.571rem]">
            <Icon name="delete_forever" tint="danger" />
            <div className="flex min-w-0 flex-1 flex-col gap-[0.286rem]">
              <OverlayTitle data-part="app.attachmentlightbox.sheetHeading">Remove attachment?</OverlayTitle>
              <Text as="p" size="md" tone="secondary" data-part="app.attachmentlightbox.sheetDetail">
                {`“${current.name}” — attachments skip the Trash.`}
              </Text>
            </div>
          </div>
          <Footer density="row" edge="none" className="min-w-0 gap-[0.429rem] px-0 py-0">
            <div className="ml-auto flex shrink-0 items-center gap-[0.429rem]">
              <Button
                data-part="app.attachmentlightbox.sheetCancel"
                variant="neutral"
                width="content"
                label="Cancel"
                onClick={() => setSheet(null)}
              />
              <Button
                data-part="app.attachmentlightbox.sheetAccept"
                variant="danger"
                width="content"
                label="Remove"
                onClick={confirmRemove}
                onKeyDown={(event) => {
                  // Enter does not confirm a removal; Space or a click does.
                  if (event.key === "Enter") event.preventDefault();
                }}
              />
            </div>
          </Footer>
        </div>
      </Overlay>
    </>
  );
}

/** The rename prompt: the base name in a field, the extension beside it, Cancel and Rename. */
function RenameForm({
  base,
  tail,
  detail,
  onSubmit,
  onCancel,
}: {
  base: string;
  tail: string;
  detail: string;
  onSubmit: (value: string) => void;
  onCancel: () => void;
}) {
  const [draft, setDraft] = React.useState(base);
  const field = React.useRef<HTMLInputElement | HTMLTextAreaElement | null>(null);

  // Focus is on the field (autoFocus); the caret goes to the end, so typed text is appended.
  React.useEffect(() => {
    const el = field.current;
    if (el) el.setSelectionRange(el.value.length, el.value.length);
  }, []);

  return (
    <div className="flex min-w-0 flex-col gap-[0.571rem] pt-[1.071rem] pr-[0.714rem] pb-[0.714rem] pl-[1.071rem]">
      <div className="flex min-w-0 items-start gap-[0.571rem]">
        <Icon name="drive_file_rename_outline" tint="accent" />
        <div className="flex min-w-0 flex-1 flex-col gap-[0.286rem]">
          <OverlayTitle data-part="app.attachmentlightbox.sheetHeading">Rename attachment</OverlayTitle>
          <Text as="p" size="md" tone="secondary" data-part="app.attachmentlightbox.sheetDetail">
            {detail}
          </Text>
        </div>
      </div>
      <div className="flex h-[1.75rem] min-w-0 items-center gap-[0.143rem] rounded-(--radius-field) border border-edge-2 bg-surface-3 pl-[0.571rem] pr-[0.571rem]">
        <Input
          ref={field}
          autoFocus
          ariaLabel="Attachment name"
          mode="bare"
          size="md"
          value={draft}
          onValueChange={setDraft}
          onSubmit={onSubmit}
          data-part="app.attachmentlightbox.renameField"
          className="h-full min-w-0 flex-1 px-0 text-[0.929rem]"
        />
        {tail !== "" && (
          <Text as="span" size="md" tone="muted" data-part="app.attachmentlightbox.sheetTail" className="shrink-0">
            {tail}
          </Text>
        )}
      </div>
      <Footer density="row" edge="none" className="min-w-0 gap-[0.429rem] px-0 py-0">
        <div className="ml-auto flex shrink-0 items-center gap-[0.429rem]">
          <Button
            data-part="app.attachmentlightbox.sheetCancel"
            variant="neutral"
            width="content"
            label="Cancel"
            onClick={onCancel}
          />
          <Button
            data-part="app.attachmentlightbox.sheetAccept"
            variant="primary"
            width="content"
            label="Rename"
            onClick={() => onSubmit(draft)}
          />
        </div>
      </Footer>
    </div>
  );
}

/** The media stage: the inline preview for a file, or the No inline preview card for a link. */
function Body({ item, kind, onOpen }: { item: Attachment; kind: FileKind; onOpen: () => void }) {
  if (kind === "image")
    return (
      <AttachmentPreview
        data-part="app.attachmentlightbox.image"
        kind="image"
        src={item.url}
        name={item.name}
        className="h-auto w-auto"
      />
    );
  if (kind === "video")
    return <AttachmentPreview kind="video" src={item.url} name={item.name} className="h-auto w-auto" />;
  if (kind === "audio")
    return <AttachmentPreview kind="audio" src={item.url} name={item.name} className="h-auto w-auto" />;
  if (kind === "pdf")
    return <AttachmentPreview kind="pdf" src={item.url} name={item.name} className="h-[34rem] w-[48rem] max-w-[90vw]" />;
  return (
    <div className="flex w-[24rem] max-w-full flex-col items-center gap-[0.714rem] rounded-(--radius-panel) bg-surface-2 px-10 py-12 text-center">
      <Icon name={KIND_ICON[kind]} size="display" weight={200} tone="muted" />
      <Text as="div" size="base" tone="primary" overflow="truncate" data-part="app.attachmentlightbox.fallbackName" className="max-w-xs">
        {item.name}
      </Text>
      <Text as="div" size="sm" tone="muted" data-part="app.attachmentlightbox.fallbackStatus">
        No inline preview
      </Text>
      <Button
        data-part="app.attachmentlightbox.fallbackOpen"
        variant="primary"
        width="content"
        icon="open_in_new"
        label="Open externally"
        className="mt-1"
        onClick={onOpen}
      />
    </div>
  );
}
