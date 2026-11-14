import { Pane } from "../components/ui/pane";

const TASKS = ["Draft the release notes", "Review the export flow", "Plan the next sprint"];
const PREVIEW = ["Preview", "Due Friday", "Two subtasks open"];
const SECTIONS = ["General", "Shortcuts", "Appearance"];

function PaneHarness(props: any) {
  // The pane takes the contract props only; emit is the harness's own channel.
  const paneProps = Object.fromEntries(Object.entries(props).filter(([key]) => key !== "emit"));
  const rows = props.as === "nav" ? SECTIONS : props.slot === "aside" ? PREVIEW : TASKS;
  return (
    <div className="flex h-screen w-full">
      <Pane {...paneProps}>
        <div className="flex flex-col gap-1 p-2 text-[0.929rem] text-content-1">
          {rows.map((text: string) => (
            <div key={text} className="rounded-(--radius-field) px-2 py-1.5">
              {text}
            </div>
          ))}
        </div>
      </Pane>
    </div>
  );
}

export default { id: "ui.pane", Component: PaneHarness, events: [] as string[] };
