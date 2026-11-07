// Generated from contracts/switch.contract.yaml by stubs.ts. Do not edit: change the contract instead.

export interface SwitchToggled { on: boolean; }
export interface SwitchProps {
  on?: boolean;
  label?: string;
  onToggled?: (event: SwitchToggled) => void;
}
export const SwitchDefaults = { on: false, label: "Dark mode" } as const;
export const Parts = { switch: "ui.switch.switch", thumb: "ui.switch.thumb" } as const;
export const Events = ["toggled"] as const;
