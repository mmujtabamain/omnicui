import { useState } from "react";
import { MenuButton } from "../components/ui/menu-button";

const DEMO_LISTS = ["Home", "Work", "Errands"];

function MenuButtonHarness({ emit, open, menu, ...props }: any) {
  // Uncontrolled unless the test sets open: then the host decides, and the trigger shows that state.
  const [own, setOwn] = useState(false);
  const controlled = open !== undefined;
  const onOpenChange = (next: boolean) => {
    if (!controlled) setOwn(next);
    emit("openChanged", { open: next });
  };
  const rows = DEMO_LISTS.map((name) => (
    <div key={name} className="flex h-[1.857rem] items-center px-3 text-[0.929rem] text-content-1">
      {name}
    </div>
  ));
  return (
    <MenuButton
      {...props}
      open={controlled ? open : own}
      onOpenChange={onOpenChange}
      menu={menu ?? rows}
    />
  );
}

export default { id: "ui.menubutton", Component: MenuButtonHarness, events: ["openChanged"] };
