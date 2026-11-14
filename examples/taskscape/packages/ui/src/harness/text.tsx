import { Text } from "../components/ui/text";

function TextHarness({ content = "", ...props }: any) {
  return <Text {...props}>{content}</Text>;
}

export default { id: "ui.text", Component: TextHarness, events: [] };
