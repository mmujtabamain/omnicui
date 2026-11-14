import { Heading } from "../components/ui/heading";

function HeadingHarness({ content = "Title", ...props }: any) {
  return <Heading {...props}>{content}</Heading>;
}

export default { id: "ui.heading", Component: HeadingHarness, events: [] };
