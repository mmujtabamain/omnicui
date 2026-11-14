import { ProgressBar } from "../components/ui/progress-bar";

// Pass only the bar's own props: emit is the omnicui bridge and must not reach the DOM root.
// Undefined props fall back to the widget's defaults.
function ProgressBarHarness({ value, max, ariaLabel, tone, className }: any) {
  return <ProgressBar value={value} max={max} ariaLabel={ariaLabel} tone={tone} className={className} />;
}

export default { id: "ui.progressbar", Component: ProgressBarHarness, events: [] };
