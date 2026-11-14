import { Spinner } from "../components/ui/spinner";

function SpinnerHarness(props: any) {
  return <Spinner {...props} />;
}

export default { id: "ui.spinner", Component: SpinnerHarness, events: [] };
