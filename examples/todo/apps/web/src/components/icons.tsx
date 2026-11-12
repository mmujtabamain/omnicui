// The few icons the app draws, as inline SVG (Material Symbols' shapes), hidden from assistive
// technology: the buttons they sit in are named by their text or aria-label.
const svg = (d: string) => () => (
  <svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><path d={d} fill="currentColor" /></svg>
);
export const PlusIcon = svg("M11 13H5v-2h6V5h2v6h6v2h-6v6h-2z");
export const CloseIcon = svg("M6.4 19 5 17.6l5.6-5.6L5 6.4 6.4 5l5.6 5.6L17.6 5 19 6.4 13.4 12l5.6 5.6-1.4 1.4-5.6-5.6z");
export const BackIcon = svg("m7.825 13 5.6 5.6L12 20l-8-8 8-8 1.425 1.4-5.6 5.6H20v2z");
export const GearIcon = svg("m9.25 22-.4-3.2q-.325-.125-.612-.3t-.563-.375L4.7 19.375l-2.75-4.75 2.575-1.95Q4.5 12.5 4.5 12.338v-.675q0-.163.025-.338L1.95 9.375l2.75-4.75 2.975 1.25q.275-.2.575-.375t.6-.3l.4-3.2h5.5l.4 3.2q.325.125.613.3t.562.375l2.975-1.25 2.75 4.75-2.575 1.95q.025.175.025.338v.674q0 .163-.05.338l2.575 1.95-2.75 4.75-2.95-1.25q-.275.2-.575.375t-.6.3l-.4 3.2zm2.8-6.5q1.45 0 2.475-1.025T15.55 12t-1.025-2.475T12.05 8.5q-1.475 0-2.488 1.025T8.55 12t1.013 2.475T12.05 15.5");
export const TaskIcon = svg("m10.6 16.6 7.05-7.05-1.4-1.4-5.65 5.65-2.85-2.85-1.4 1.4zM5 21q-.825 0-1.412-.587T3 19V5q0-.825.588-1.412T5 3h14q.825 0 1.413.588T21 5v14q0 .825-.587 1.413T19 21z");
