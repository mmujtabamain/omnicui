import { Badge } from "../components/ui/badge";

function BadgeHarness({ emit, ...props }: any) {
  void emit; // read-only: the badge emits no events
  return <Badge {...props} />;
}

export default { id: "ui.badge", Component: BadgeHarness, events: [] };
