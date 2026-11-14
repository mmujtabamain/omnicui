// The Empty Trash confirmation: a card over a soft scrim, the same look as the app's sheets. Cancel
// is the first control, so it takes focus when the card opens (decision 13). Enter does not confirm
// a destructive sheet (decision 10); Space or a click does. Escape cancels.
import { useId } from "react";
import { Button } from "@taskscape/ui/button";
import { Footer } from "@taskscape/ui/footer";
import { Icon } from "@taskscape/ui/icon";
import { Overlay, OverlayTitle } from "@taskscape/ui/overlay";
import { Scrim } from "@taskscape/ui/scrim";
import { Text } from "@taskscape/ui/text";

export interface EmptySheetProps {
  /** Every trashed item, as the count the sheet names. */
  count: number;
  onCancel: () => void;
  onConfirm: () => void;
}

export function EmptySheet({ count, onCancel, onConfirm }: EmptySheetProps) {
  const detailId = useId();
  const detail = `${count} ${count === 1 ? "task" : "tasks"} deleted for good. No undo.`;

  return (
    <>
      <Scrim dim="soft" className="animate-[fade-in_var(--duration-fast)_var(--ease-standard)_both]" />
      <Overlay
        data-part="app.trash.emptySheet"
        open
        onOpenChange={(open) => {
          if (!open) onCancel();
        }}
        aria-describedby={detailId}
        className="w-[min(26rem,100%)] overflow-hidden"
      >
        <div className="flex min-w-0 flex-col pt-[0.571rem] pr-[0.714rem] pb-[0.714rem] pl-[0.714rem]">
          <div className="flex min-w-0 items-center gap-[0.357rem]">
            <Icon name="delete_forever" tint="danger" data-part="app.trash.emptySheetGlyph" />
            <OverlayTitle data-part="app.trash.emptySheetTitle" className="min-w-0 flex-1 font-display text-pretty">
              Empty Trash?
            </OverlayTitle>
          </div>
          <Text
            as="div"
            id={detailId}
            data-part="app.trash.emptySheetDetail"
            size="caption"
            tone="secondary"
            className="mt-[0.429rem]"
          >
            {detail}
          </Text>
        </div>
        <Footer density="row" edge="none" className="min-w-0 gap-[0.357rem] border-t border-edge-1 px-[0.714rem] py-[0.643rem]">
          <div className="ml-auto grid grid-cols-2 gap-[0.357rem]">
            <Button
              data-part="app.trash.cancelEmpty"
              variant="neutral"
              size="lg"
              width="content"
              label="Cancel"
              className="min-w-[7.143rem]"
              onClick={() => onCancel()}
            />
            <Button
              data-part="app.trash.confirmEmpty"
              variant="danger"
              size="lg"
              width="content"
              label="Empty Trash"
              className="min-w-[7.143rem]"
              onClick={() => onConfirm()}
              onKeyDown={(event) => {
                // Destructive confirmations: Enter does not confirm (decision 10).
                if (event.key === "Enter") event.preventDefault();
              }}
            />
          </div>
        </Footer>
      </Overlay>
    </>
  );
}
