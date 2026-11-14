import { ConfirmButton } from "../components/ui/confirm-button";

function ConfirmButtonHarness({ emit, tone, label, confirmLabel, ...props }: any) {
  return (
    <ConfirmButton
      tone={tone}
      label={label}
      confirmLabel={confirmLabel}
      onConfirm={() => emit("confirmed", {})}
      {...props}
    />
  );
}

export default { id: "ui.confirmbutton", Component: ConfirmButtonHarness, events: ["confirmed"] };
