import { HotkeyHint } from "../components/ui/hotkey-hint";

function HotkeyHintHarness({ emit, ...props }: any) {
  void emit; // the widget is display only and emits no events
  return <HotkeyHint {...props} />;
}

export default { id: "ui.hotkeyhint", Component: HotkeyHintHarness, events: [] };
