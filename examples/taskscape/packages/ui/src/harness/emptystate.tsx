import { EmptyState } from "../components/ui/empty-state";

function EmptyStateHarness(harnessProps: any) {
  // emit belongs to the harness, not the widget (which has no events), so it is not spread onto the DOM.
  const props = { ...harnessProps, emit: undefined };
  // A pane fills the task pane, so the demo mount gives it the viewport height to fill.
  if (props.scope === "pane")
    return (
      <div className="h-screen w-full">
        <EmptyState {...props} />
      </div>
    );
  return <EmptyState {...props} />;
}

export default { id: "ui.emptystate", Component: EmptyStateHarness, events: [] };
