import { CharacterCounter } from "../components/ui/character-counter";

function CharacterCounterHarness({ count, limit, nearRatio }: any) {
  return <CharacterCounter count={count} limit={limit} nearRatio={nearRatio} />;
}

export default { id: "ui.charactercounter", Component: CharacterCounterHarness, events: [] };
