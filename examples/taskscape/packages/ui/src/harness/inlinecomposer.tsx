import { InlineComposer } from "../components/ui/inline-composer";

function InlineComposerHarness({ emit, ...props }: any) {
  return (
    <InlineComposer
      {...props}
      onSubmit={(title: string) => emit("submitted", { title })}
      onDismiss={() => emit("dismissed", {})}
    />
  );
}

export default { id: "ui.inlinecomposer", Component: InlineComposerHarness, events: ["submitted", "dismissed"] };
