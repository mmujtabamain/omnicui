import * as TabsPrimitive from "@radix-ui/react-tabs";
import { TabPanel } from "../components/ui/tab-panel";

// Sized for the 640 x 520 desktop window: with scroll on, the pane overflows by about one
// row, so one scroll step (about 100px) reaches the bottom edge, as the atBottom tests need.
const SECTIONS = [
  { title: "General", rows: [["Launch at login", "On"], ["Show in menu bar", "On"], ["Language", "English"]] },
  { title: "Startup", rows: [["Open last workspace", "On"], ["Restore windows", "Off"], ["Startup sound", "Off"]] },
  { title: "Appearance", rows: [["Theme", "System"], ["Density", "Comfortable"], ["Reduce motion", "Off"]] },
  { title: "Notifications", rows: [["Task reminders", "On"]] },
];

function Group({ title, rows }: { title: string; rows: string[][] }) {
  return (
    <section className="mb-5">
      <h3 className="mb-1.5 text-[0.786rem] font-medium text-content-3">{title}</h3>
      <div className="rounded-(--radius-panel) border border-edge-1 bg-surface-3">
        {rows.map(([label, value], i) => (
          <div
            key={label}
            className={`flex h-[2.5rem] items-center justify-between px-3 text-[0.929rem] ${i > 0 ? "border-t border-edge-1" : ""}`}
          >
            <span className="text-content-1">{label}</span>
            <span className="text-content-2">{value}</span>
          </div>
        ))}
      </div>
    </section>
  );
}

function TabPanelHarness({ emit, value = "", scroll, tone, ...props }: any) {
  const groups = scroll === false ? SECTIONS.slice(0, 2) : SECTIONS;
  return (
    <TabsPrimitive.Root value={value} className="h-screen">
      <TabPanel
        {...props}
        value={value}
        scroll={scroll}
        tone={tone}
        onScrolled={(fields: { atBottom: boolean }) => emit("scrolled", { atBottom: fields.atBottom })}
      >
        {groups.map((g) => (
          <Group key={g.title} title={g.title} rows={g.rows} />
        ))}
      </TabPanel>
    </TabsPrimitive.Root>
  );
}

export default { id: "ui.tabpanel", Component: TabPanelHarness, events: ["scrolled"] };
