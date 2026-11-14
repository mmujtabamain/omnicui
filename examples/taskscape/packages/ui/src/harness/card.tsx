import { Card } from "../components/ui/card";

function CardHarness({ emit, discardable = false, asChild = false, label = "", ...props }: any) {
  const onDiscard = discardable ? () => emit("discarded", { label }) : undefined;
  if (asChild)
    return (
      <Card asChild label={label} onDiscard={onDiscard} {...props}>
        <section className="px-3 py-2 text-[0.929rem] text-content-1">Launch at login</section>
      </Card>
    );
  return <Card label={label} onDiscard={onDiscard} {...props} />;
}

export default { id: "ui.card", Component: CardHarness, events: ["discarded"] };
