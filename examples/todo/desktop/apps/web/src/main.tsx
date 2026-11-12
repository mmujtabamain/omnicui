// The todo web app: `npm start` builds it and serves it on 127.0.0.1. It starts with the "three"
// fixture tasks, as the Android app does.
import { createRoot } from "react-dom/client";
import { TodoApp } from "./components/index.tsx";

document.body.style.margin = "0";
createRoot(document.getElementById("root")!).render(<TodoApp seed="three" />);
