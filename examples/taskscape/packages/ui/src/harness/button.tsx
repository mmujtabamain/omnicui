import { useRef } from "react";
import { Button } from "../components/ui/button";

function ButtonHarness({ emit, asChild, label, ...props }: any) {
  const count = useRef(0);
  const activated = () => emit("activated", { count: ++count.current });
  if (asChild)
    return (
      <Button asChild {...props}>
        <button type="button" onClick={activated}>{label}</button>
      </Button>
    );
  return <Button label={label} {...props} onClick={activated} />;
}

export default { id: "ui.button", Component: ButtonHarness, events: ["activated"] };
