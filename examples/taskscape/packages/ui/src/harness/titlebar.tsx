import { useState } from "react";
import { TitleBar } from "../components/ui/title-bar";

function TitleBarHarness({ emit, ...props }: any) {
  const [menuOpen, setMenuOpen] = useState<boolean>(Boolean(props.projectMenuOpen));
  const [maximized, setMaximized] = useState(false);
  const toggleMaximize = () => {
    const next = !maximized;
    setMaximized(next);
    emit("maximizeToggled", { maximized: next });
  };
  return (
    <TitleBar
      {...props}
      projectMenuOpen={menuOpen}
      onProjectMenuOpenChange={(open: boolean) => {
        setMenuOpen(open);
        emit("projectMenuChanged", { open });
      }}
      onMaximizeToggle={toggleMaximize}
    />
  );
}

export default {
  id: "ui.titlebar",
  Component: TitleBarHarness,
  events: ["projectMenuChanged", "maximizeToggled"],
};
