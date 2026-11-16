// One row of the project menu: the project name (a menuitem), always-visible rename and delete buttons beside
// it, and a More button whose menu holds the same two actions. The menu itself lives in TitleBar.tsx.
import * as React from "react";
import { Button } from "@taskscape/ui/button";
import { Icon } from "@taskscape/ui/icon";
import { Text } from "@taskscape/ui/text";
import type { Project } from "../../api/types";

export interface ProjectRowProps {
  project: Project;
  /** The active project: check mark and aria-selected. */
  active: boolean;
  /** Test part names; only the first and second rows carry them. */
  part?: string;
  renamePart?: string;
  deletePart?: string;
  onPick(): void;
  onRename(): void;
  onDelete(): void;
}

export function ProjectRow({ project, active, part, renamePart, deletePart, onPick, onRename, onDelete }: ProjectRowProps) {
  // The whole row takes the hover highlight; rename and delete show inside it on hover or keyboard focus (F02-010),
  // as in the reference. No row menu: the two actions are the row's own.
  return (
    <div className="group/row flex items-center rounded-(--radius-field) pr-[0.357rem] hover:bg-wash-2 focus-within:bg-wash-2">
      <div
        role="menuitem"
        aria-selected={active}
        tabIndex={0}
        data-part={part}
        onClick={onPick}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            onPick();
          }
        }}
        className="flex h-[2.286rem] min-w-0 flex-1 cursor-default items-center gap-[0.357rem] rounded-(--radius-field) pr-2 pl-[0.643rem] outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1"
      >
        <span className="flex w-[1.071rem] shrink-0 items-center justify-center">
          {active && <Icon name="check" size={15} weight={400} tone="accent" />}
        </span>
        <Text as="span" size="md" overflow="truncate" className="min-w-0 text-[0.964rem]">
          {project.name}
        </Text>
      </div>
      <Button
        data-part={renamePart}
        ariaLabel="Rename project"
        description="Rename project"
        icon="edit"
        iconSize={16}
        shape="icon"
        size="sm"
        variant="ghost"
        tone="muted"
        reveal="row"
        onClick={onRename}
      />
      <Button
        data-part={deletePart}
        ariaLabel="Delete project"
        description="Delete project"
        icon="delete"
        iconSize={16}
        shape="icon"
        size="sm"
        variant="ghost"
        tone="danger"
        reveal="row"
        onClick={onDelete}
      />
    </div>
  );
}
