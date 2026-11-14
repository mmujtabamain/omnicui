import { useState } from "react";
import { Row } from "../components/ui/row";
import { Button } from "../components/ui/button";

/**
 * Demo content: a title, and the Open control on multi-select rows (inset, md). Open shows on
 * hover and on keyboard focus only, not on selection. A press toggles its active mark.
 */
function RowHarness({ shape, density, tone, selected, indent }: any) {
  const [opened, setOpened] = useState(false);
  const multi = (shape ?? "inset") === "inset" && (density ?? "md") === "md";
  return (
    <Row shape={shape} density={density} tone={tone} selected={selected} indent={indent}>
      <span className="min-w-0 flex-1 truncate text-content-1">Write the quarterly report</span>
      {multi && (
        <Button
          data-slot="open"
          reveal="always"
          className="ml-auto opacity-0 group-hover/row:opacity-100 focus-visible:opacity-100"
          size="sm"
          variant="neutral"
          width="content"
          label="Open"
          active={opened}
          onClick={() => setOpened((v) => !v)}
        />
      )}
    </Row>
  );
}

export default { id: "ui.row", Component: RowHarness, events: [] };
