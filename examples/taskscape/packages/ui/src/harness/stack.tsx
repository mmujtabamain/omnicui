import { Stack } from "../components/ui/stack";

function StackHarness({ emit, asChild, ...props }: any) {
  // Stack emits no events, so emit is kept out of the DOM props and never called.
  void emit;
  // Two plain, unstyled fixture children: One, then Two.
  const body = (
    <>
      <div data-slot="first">One</div>
      <div data-slot="second">Two</div>
    </>
  );
  if (asChild)
    return (
      <Stack asChild {...props}>
        <div>{body}</div>
      </Stack>
    );
  return <Stack {...props}>{body}</Stack>;
}

export default { id: "ui.stack", Component: StackHarness, events: [] };
