// The harness's emit(name, fields): the contract's event name and exactly its declared fields.
export type Emit = (name: string, fields: Record<string, unknown>) => void;

export const noEmit: Emit = () => undefined;
