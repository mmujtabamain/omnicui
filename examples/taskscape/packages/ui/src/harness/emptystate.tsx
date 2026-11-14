import { EmptyState } from "../components/ui/empty-state";

function EmptyStateHarness(props: any) {
  const { emit, ...rest } = props;
  void emit;
  return <EmptyState {...rest} />;
}

export default { id: "ui.emptystate", Component: EmptyStateHarness, events: [] as string[] };
