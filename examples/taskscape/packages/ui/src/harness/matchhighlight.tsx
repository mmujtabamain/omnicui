import { MatchHighlight } from "../components/ui/match-highlight";

function MatchHighlightHarness(props: any) {
  return <MatchHighlight {...props} />;
}

export default { id: "ui.matchhighlight", Component: MatchHighlightHarness, events: [] };
