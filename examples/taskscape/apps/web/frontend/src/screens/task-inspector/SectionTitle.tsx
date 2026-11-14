import * as React from "react";
import { Text } from "@taskscape/ui/text";

/** An eyebrow section label with its hairline and trailing controls. The label is the heading. */
export function SectionTitle({
  label,
  part,
  trailing,
  rule = true,
}: {
  label: string;
  part: string;
  trailing?: React.ReactNode;
  rule?: boolean;
}) {
  return (
    <div className="flex min-w-0 items-center gap-2">
      <Text
        as="div"
        role="heading"
        aria-level={3}
        data-part={part}
        size="xs"
        weight="semibold"
        tone="muted"
        className="shrink-0 tracking-[0.08em] [font-variant-caps:all-small-caps]"
      >
        {label}
      </Text>
      {rule && <span aria-hidden="true" className="block h-px min-w-4 flex-1 bg-edge-2" />}
      {trailing && <div className="ml-auto flex shrink-0 items-center gap-1">{trailing}</div>}
    </div>
  );
}
