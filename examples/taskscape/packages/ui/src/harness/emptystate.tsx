import { EmptyState } from "../components/ui/empty-state";

function EmptyStateHarness(props: any) {
  const { emit, ...rest } = props;
  void emit;
  return (
    <div className="flex h-dvh w-full flex-col">
      <EmptyState {...rest} />
    </div>
  );
}

export default { id: "ui.emptystate", Component: EmptyStateHarness, events: [] as string[] };
