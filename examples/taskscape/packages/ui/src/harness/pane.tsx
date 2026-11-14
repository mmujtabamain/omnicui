import { Button } from "../components/ui/button";
import { Pane } from "../components/ui/pane";

const TASKS = [
  "Draft the release notes",
  "Review the onboarding copy",
  "Ship the preview build",
  "Book the design review",
  "Archive last quarter's boards",
];

const SETTINGS = [
  { icon: "tune", label: "General" },
  { icon: "keyboard", label: "Shortcuts" },
  { icon: "palette", label: "Appearance" },
  { icon: "notifications", label: "Notifications" },
];

function PaneHarness(props: any) {
  const { slot = "primary", as = "div" } = props;
  let content;
  if (as === "nav") {
    content = (
      <div className="flex flex-col gap-0.5 p-2">
        {SETTINGS.map((item) => (
          <Button key={item.label} block icon={item.icon} label={item.label} />
        ))}
      </div>
    );
  } else if (slot === "aside") {
    content = (
      <div className="flex flex-col gap-2 p-3">
        <span className="text-content-1 font-medium">Preview</span>
        <span className="text-content-2">Ship the preview build</span>
        <span className="text-content-3">Due Friday, assigned to you</span>
      </div>
    );
  } else {
    content = (
      <div className="flex min-h-0 min-w-0 flex-1 flex-col gap-0.5 p-2">
        {TASKS.map((task) => (
          <Button key={task} block icon="check_box_outline_blank" label={task} />
        ))}
      </div>
    );
  }
  return (
    <div className="flex h-screen w-screen bg-surface-0">
      <Pane {...props}>{content}</Pane>
    </div>
  );
}

export default { id: "ui.pane", Component: PaneHarness, events: [] };
