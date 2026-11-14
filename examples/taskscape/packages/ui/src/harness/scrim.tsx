import { Scrim } from "../components/ui/scrim";

function ScrimHarness({ emit, ...props }: any) {
  return <Scrim {...props} onDismiss={() => emit("dismissed", {})} />;
}

export default { id: "ui.scrim", Component: ScrimHarness, events: ["dismissed"] };
