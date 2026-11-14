import { Heading } from "../components/ui/heading";

function HeadingHarness({ emit, content = "Title", ...props }: any) {
  void emit; // a heading emits no events
  return <Heading {...props}>{content}</Heading>;
}

export default { id: "ui.heading", Component: HeadingHarness, events: [] };
