import { HotkeyHint } from "../components/ui/hotkey-hint";

function HotkeyHintHarness(props: any) {
  return <HotkeyHint {...props} />;
}

export default { id: "ui.hotkeyhint", Component: HotkeyHintHarness, events: [] };
