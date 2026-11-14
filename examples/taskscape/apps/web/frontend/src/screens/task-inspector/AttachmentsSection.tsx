import * as React from "react";
import { AttachmentTile } from "@taskscape/ui/attachment-tile";
import { Button } from "@taskscape/ui/button";
import { Separator } from "@taskscape/ui/separator";
import { Text } from "@taskscape/ui/text";
import type { Attachment } from "../../api/types";
import type { Emit } from "./events";
import { SectionTitle } from "./SectionTitle";

const P = "app.taskinspector.";
const IMAGE = /\.(png|jpe?g|gif|webp|svg|bmp|avif)$/i;

/** The Attachments section: File and Link in the header, then the tiles or the empty box. */
export function AttachmentsSection({
  attachments,
  emit,
  onOpen,
}: {
  attachments: Attachment[];
  emit: Emit;
  onOpen: (attachment: Attachment) => void;
}) {
  return (
    <div className="flex min-w-0 flex-col gap-2">
      <SectionTitle
        label="Attachments"
        part={`${P}attachmentsHeading`}
        rule={false}
        trailing={
          <>
            <Button
              data-part={`${P}attachLink`}
              ariaLabel="Add a link"
              description="Add a link"
              label="Link"
              icon="add_link"
              iconSize={14}
              variant="ghost"
              size="sm"
              width="content"
              truncate={false}
              tone="muted"
              className="font-semibold"
              onClick={() => emit("linkRequested", { where: "attachments" })}
            />
            <Button
              data-part={`${P}attachFile`}
              label="File"
              icon="note_add"
              iconSize={14}
              variant="ghost"
              size="sm"
              width="content"
              truncate={false}
              tone="muted"
              className="font-semibold"
              onClick={() => emit("attachRequested", { kind: "file" })}
            />
          </>
        }
      />
      {attachments.length === 0 ? (
        <div className="flex h-[5rem] min-w-0 items-stretch overflow-hidden rounded-(--radius-control) border border-dashed border-edge-2 bg-surface-3">
          <Button
            data-part={`${P}emptyCopyFile`}
            label="Copy a file"
            icon="note_add"
            iconSize={18}
            variant="ghost"
            size="md"
            tone="muted"
            description="Copy a file into this task"
            className="h-auto min-w-0 flex-1 flex-col gap-1 rounded-none font-semibold"
            onClick={() => emit("attachRequested", { kind: "file" })}
          />
          <Separator orientation="vertical" length="fill" />
          <Button
            data-part={`${P}emptyAddLink`}
            ariaLabel="Add a link"
            label="Add a link"
            icon="add_link"
            iconSize={18}
            variant="ghost"
            size="md"
            tone="muted"
            description="Link a file or URL without copying"
            className="h-auto min-w-0 flex-1 flex-col gap-1 rounded-none font-semibold"
            onClick={() => emit("linkRequested", { where: "emptyBox" })}
          />
        </div>
      ) : (
        <div className="flex min-w-0 flex-wrap gap-[0.357rem]">
          {attachments.map((attachment) => {
            const remote = attachment.kind === "link";
            const image = !remote && IMAGE.test(attachment.name);
            const part = remote ? `${P}attachmentTileLink` : image ? `${P}attachmentTileImage` : undefined;
            return (
              <div key={attachment.id} className="flex w-[6.714rem] min-w-0 flex-col">
                <AttachmentTile
                  data-part={part}
                  kind={image ? "image" : "link"}
                  name={attachment.name}
                  thumbnailSrc={image ? attachment.url : undefined}
                  onOpen={() => onOpen(attachment)}
                />
                <Text as="div" size="xs" tone="muted" overflow="truncate" className="mt-1 w-full text-center">
                  {attachment.name}
                </Text>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
