import { AppHarness } from "./lib/AppHarness";
import { Crash } from "../screens/crash/Crash";

function CrashHarness({ message = "Something broke" }: { message?: string }) {
  return <AppHarness><Crash error={new Error(message)} /></AppHarness>;
}

export default { id: "app.crash", Component: CrashHarness, events: [] };
