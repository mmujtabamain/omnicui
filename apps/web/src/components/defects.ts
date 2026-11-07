// Deliberate defects, switched on with OMNICUI_DEFECTS=name,name… so that every check failure
// can be caused on purpose. With none set, the components are correct.
const active = new Set<string>(((globalThis as { __OMNICUI_DEFECTS__?: string }).__OMNICUI_DEFECTS__ ?? "").split(",").filter(Boolean));
export const defect = (name: string) => active.has(name);
