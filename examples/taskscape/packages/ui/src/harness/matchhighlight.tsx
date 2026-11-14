import { MatchHighlight } from "../components/ui/match-highlight";

function MatchHighlightHarness(props: any) {
  return <MatchHighlight query={props.query} text={props.text} />;
}

export default { id: "ui.matchhighlight", Component: MatchHighlightHarness, events: [] };
