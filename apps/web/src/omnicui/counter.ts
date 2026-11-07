// Generated from omnicui/contracts/counter.contract.yaml by stubs.ts. Do not edit: change the contract instead.

export interface CounterChanged { count: number; }
export interface CounterProps {
  start?: number;
  onChanged?: (event: CounterChanged) => void;
}
export const CounterDefaults = { start: 0 } as const;
export const Parts = { value: "ui.counter.value", increment: "ui.counter.increment" } as const;
export const Events = ["changed"] as const;
