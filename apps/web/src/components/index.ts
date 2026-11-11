// The app's components, as a library: what the app itself uses, and what the omnicui web target imports.
import "./components.css";
export { Counter } from "./Counter.tsx";
export { Switch } from "./Switch.tsx";
// A second design of the same components (same behaviour, another look), for the webalt target.
import "./redesign/redesign.css";
export { RedesignedCounter } from "./redesign/Counter.tsx";
export { RedesignedSwitch } from "./redesign/Switch.tsx";
