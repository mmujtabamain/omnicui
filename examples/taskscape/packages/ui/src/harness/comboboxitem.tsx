import { useEffect, useState, type KeyboardEvent } from "react";
import { ComboboxItem } from "../components/ui/combobox-item";

/**
 * The host text field and list: the input forwards arrow keys to the single row, Enter and
 * Tab (mention) commit the active row, and the row's pointer events drive the same state.
 */
function ComboboxItemHarness({ emit, context = "palette", active: activeProp = false, id = "", ...props }: any) {
  const [active, setActive] = useState<boolean>(activeProp);
  useEffect(() => {
    setActive(activeProp);
  }, [activeProp]);

  const highlight = () => {
    if (active) return;
    setActive(true);
    emit("highlighted", { id });
  };
  const pick = () => emit("picked", { id });

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    switch (event.key) {
      case "ArrowDown":
        event.preventDefault();
        highlight();
        break;
      case "ArrowUp":
        event.preventDefault();
        if (active) setActive(false);
        else highlight();
        break;
      case "Enter":
        event.preventDefault();
        if (active) pick();
        break;
      case "Tab":
        if (context === "mention" && active) {
          event.preventDefault();
          pick();
        }
        break;
      default:
        break;
    }
  };

  return (
    <div className="w-[34.857rem] overflow-hidden rounded-(--radius-panel) bg-surface-2 shadow-menu">
      <input
        data-slot="input"
        type="text"
        placeholder="Type a command or a name"
        aria-activedescendant={active && id ? id : undefined}
        className="h-[2.143rem] w-full bg-transparent px-[0.643rem] text-[0.929rem] leading-[1.393rem] text-content-1 outline-none placeholder:text-content-3"
        onKeyDown={onKeyDown}
      />
      <div role="listbox" aria-label="Options">
        <ComboboxItem
          {...props}
          context={context}
          id={id}
          active={active}
          onSelect={pick}
          onHighlight={highlight}
        />
      </div>
    </div>
  );
}

export default { id: "ui.comboboxitem", Component: ComboboxItemHarness, events: ["picked", "highlighted"] };
