import { Button } from "../components/ui/button";
import { PaneFooter } from "../components/ui/pane-footer";
import { Separator } from "../components/ui/separator";

function PaneFooterHarness({ emit, done = 2, total = 8, percent, ...props }: any) {
  // The footer emits nothing; its children own their own keys.
  void emit;
  return (
    <PaneFooter done={done} total={total} percent={percent} {...props}>
      <Button variant="ghost" size="footer" icon="swap_vert" label="Sort" />
      <Separator orientation="vertical" length="xs" emphasis="adjacent" />
      <Button variant="ghost" size="footer" icon="filter_list" label="Filter" />
    </PaneFooter>
  );
}

export default { id: "ui.panefooter", Component: PaneFooterHarness, events: [] };
