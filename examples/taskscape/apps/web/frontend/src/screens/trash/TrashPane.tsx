// The Trash pane (contract app.trash): the trashed root tasks in the aside. Each row has Restore and
// Delete permanently (the latter immediate, decision 12); the strip holds Restore all and Empty, and
// Empty asks first. A restore or purge that hits an item changed elsewhere reloads the list and shows
// a short inline notice at the top of the pane (decision 14).
import { useState, type ReactNode } from "react";
import { Button } from "@taskscape/ui/button";
import { EmptyState } from "@taskscape/ui/empty-state";
import { Heading } from "@taskscape/ui/heading";
import { Icon } from "@taskscape/ui/icon";
import { Row } from "@taskscape/ui/row";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { useTrash, useWrite } from "../../api/queries";
import type { Id, TrashItem } from "../../api/types";
import { EmptySheet } from "./EmptySheet";
import { relativeTime } from "./relative";

export interface TrashPaneProps {
  /** The red close light was pressed. */
  onClose?: () => void;
  /** A restore call succeeded for this many root ids. */
  onRestored?: (count: number) => void;
  /** A purge or empty call succeeded for this many items. */
  onPurged?: (count: number) => void;
}

/** The part names of the two demo rows, found by title; any other row carries no name. */
interface NamedRow {
  row: string;
  meta: string;
  restore: string;
  purge: string;
}

const NAMED_ROWS: Record<string, NamedRow> = {
  "Call the printer about samples": {
    row: "rowPrinter", meta: "metaPrinter", restore: "restorePrinter", purge: "purgePrinter",
  },
  "Print the old flyer": {
    row: "rowFlyer", meta: "metaFlyer", restore: "restoreFlyer", purge: "purgeFlyer",
  },
};

const CHANGED_ELSEWHERE = "That changed elsewhere, so the list has been refreshed.";
const EMPTY_FAILED = "Could not empty the trash. The list has been refreshed.";

function part(name: string | undefined): string | undefined {
  return name ? `app.trash.${name}` : undefined;
}

interface TrashRowProps {
  item: TrashItem;
  first: boolean;
  onRestore: (id: Id) => void;
  onPurge: (id: Id) => void;
}

/**
 * A row action's slot. When the button is also the first row's positional part, the row's name
 * (forwardPart) goes on this wrapper: one element cannot carry two parts. The wrapper takes focus
 * and passes Enter and Space on to its button, so a keypress does what a press does.
 */
function ActionSlot({ forwardPart, children }: { forwardPart?: string; children: ReactNode }) {
  if (!forwardPart) return <span className="flex">{children}</span>;
  return (
    <span
      data-part={forwardPart}
      tabIndex={-1}
      className="flex outline-none"
      onKeyDown={(event) => {
        if (event.target !== event.currentTarget) return;
        if (event.key !== "Enter" && event.key !== " ") return;
        event.preventDefault();
        event.currentTarget.querySelector("button")?.click();
      }}
    >
      {children}
    </span>
  );
}

/** One trashed root. The first row carries the positional parts (row, restoreItem, purgeItem). */
function TrashRow({ item, first, onRestore, onPurge }: TrashRowProps) {
  const named = NAMED_ROWS[item.title];
  const restorePart = first ? "app.trash.restoreItem" : part(named?.restore);
  const purgePart = first ? "app.trash.purgeItem" : part(named?.purge);
  const sharedRestore = first && named ? part(named.restore) : undefined;
  const sharedPurge = first && named ? part(named.purge) : undefined;
  return (
    <div data-part={first ? "app.trash.row" : undefined}>
      <Row data-part={part(named?.row)} className="px-[0.429rem]">
        <div className="flex min-w-0 flex-1 flex-col">
          <Text as="div" size="base" overflow="truncate">
            {item.title}
          </Text>
          <Text as="div" size="xs" tone="muted" overflow="truncate" data-part={part(named?.meta)}>
            {`${item.list_name} · deleted ${relativeTime(item.deleted_at)}`}
          </Text>
        </div>
        <div className="flex shrink-0 items-center gap-[0.5rem]">
          <ActionSlot forwardPart={sharedRestore}>
            <Button
              data-part={restorePart}
              variant="ghost"
              tone="muted"
              shape="icon"
              size="md"
              width="content"
              reveal="row"
              className="[@media(hover:none)]:opacity-100"
              icon="restore"
              iconSize={16}
              ariaLabel="Restore"
              description="Restore"
              onClick={() => onRestore(item.id)}
            />
          </ActionSlot>
          <ActionSlot forwardPart={sharedPurge}>
            <Button
              data-part={purgePart}
              variant="danger"
              shape="icon"
              size="md"
              width="content"
              reveal="row"
              className="[@media(hover:none)]:opacity-100"
              icon="delete_forever"
              iconSize={16}
              ariaLabel="Delete permanently"
              description="Delete permanently"
              onClick={() => onPurge(item.id)}
            />
          </ActionSlot>
        </div>
      </Row>
    </div>
  );
}

export function TrashPane({ onClose, onRestored, onPurged }: TrashPaneProps) {
  const trash = useTrash();
  const restore = useWrite(ep.trash.restore);
  const purge = useWrite(ep.trash.purge);
  const empty = useWrite(ep.trash.empty);
  const [notice, setNotice] = useState("");
  const [sheetOpen, setSheetOpen] = useState(false);

  const items = trash.data ?? [];
  // Until the first load returns, the empty state reads Loading (F07-007); later refetches keep the list.
  const loading = trash.isPending;

  const restoreIds = async (ids: Id[]) => {
    setNotice("");
    try {
      await restore.mutateAsync([ids]);
      onRestored?.(ids.length);
    } catch {
      setNotice(CHANGED_ELSEWHERE);
    }
  };

  const purgeIds = async (ids: Id[]) => {
    setNotice("");
    try {
      await purge.mutateAsync([ids]);
      onPurged?.(ids.length);
    } catch {
      setNotice(CHANGED_ELSEWHERE);
    }
  };

  const confirmEmpty = async () => {
    const count = items.length;
    setSheetOpen(false);
    setNotice("");
    try {
      await empty.mutateAsync([]);
      onPurged?.(count);
    } catch {
      setNotice(EMPTY_FAILED);
    }
  };

  return (
    <div
      data-part="app.trash.pane"
      className="flex h-full min-h-0 w-full min-w-0 flex-col border-l border-edge-2 bg-surface-1 text-content-1"
    >
      <div className="flex h-[3.286rem] shrink-0 items-center border-b border-edge-1">
        <div className="flex shrink-0 items-center gap-[0.5rem] pl-[1.286rem]">
          <Button
            data-part="app.trash.closeControl"
            variant="ghost"
            shape="round"
            size="sm"
            width="content"
            ariaLabel="Close Trash"
            description="Close Trash"
            className="h-[1rem] w-[1rem] bg-danger-500 p-0"
            onClick={() => onClose?.()}
          />
          {/* Minimize and Full Screen are dropped in this pane (F07-082); the space keeps the title in place. */}
          <span aria-hidden="true" className="size-[1rem]" />
          <span aria-hidden="true" className="size-[1rem]" />
        </div>
        <div className="flex min-w-0 flex-1 items-center gap-[0.357rem] pr-[0.714rem] pl-[1.357rem]">
          <Icon name="delete" size="lg" weight={400} tone="secondary" />
          <Heading as="h2" size="sm" data-part="app.trash.title">
            Trash
          </Heading>
        </div>
      </div>

      {notice !== "" && (
        <div data-part="app.trash.notice" role="status" className="shrink-0 border-b border-edge-1 px-[0.643rem] py-[0.357rem]">
          <Text as="div" size="sm" tone="danger">
            {notice}
          </Text>
        </div>
      )}

      {items.length > 0 && (
        <div
          data-part="app.trash.actionStrip"
          className="flex shrink-0 items-center gap-[0.429rem] border-b border-edge-1 px-[0.643rem] py-[0.5rem]"
        >
          <Button
            data-part="app.trash.restoreAll"
            variant="ghost"
            tone="muted"
            size="md"
            width="content"
            icon="restore"
            iconSize={14}
            label="Restore all"
            onClick={() => void restoreIds(items.map((item) => item.id))}
          />
          <Button
            data-part="app.trash.emptyTrash"
            variant="danger"
            size="md"
            width="content"
            className="ml-auto"
            icon="delete_forever"
            iconSize={14}
            label="Empty"
            onClick={() => setSheetOpen(true)}
          />
        </div>
      )}

      <div className="min-h-0 flex-1 overflow-y-auto p-[0.571rem]">
        {loading && (
          <EmptyState data-part="app.trash.emptyState" title="Loading…" loading icon="delete" iconSize={28} />
        )}
        {!loading && items.length === 0 && (
          <EmptyState data-part="app.trash.emptyState" title="Trash is empty" icon="delete" iconSize={28} />
        )}
        {items.length > 0 && (
          <div data-part="app.trash.list" className="flex flex-col">
            {items.map((item, index) => (
              <TrashRow
                key={item.id}
                item={item}
                first={index === 0}
                onRestore={(id) => void restoreIds([id])}
                onPurge={(id) => void purgeIds([id])}
              />
            ))}
          </div>
        )}
      </div>

      {sheetOpen && (
        <EmptySheet
          count={items.length}
          onCancel={() => setSheetOpen(false)}
          onConfirm={() => void confirmEmpty()}
        />
      )}
    </div>
  );
}
