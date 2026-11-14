import { StatusIndicator } from "../components/ui/status-indicator";

function StatusIndicatorHarness({ emit, ...props }: any) {
  void emit;
  return <StatusIndicator {...props} />;
}

export default { id: "ui.statusindicator", Component: StatusIndicatorHarness, events: [] };
