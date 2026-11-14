import { ProgressBar } from "../components/ui/progress-bar";

// emit is the omnicui bridge, not a prop of the bar: keep it off the DOM root.
function ProgressBarHarness({ emit, ...props }: any) {
  return <ProgressBar {...props} />;
}

export default { id: "ui.progressbar", Component: ProgressBarHarness, events: [] };
