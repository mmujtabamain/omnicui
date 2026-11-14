// Every screen the app routes to. Each lives in its own folder and draws only with @taskscape/ui.
import { Crash } from "./crash/Crash";
import { MainWindow } from "./main-window/MainWindow";
import { SignIn } from "./sign-in/SignIn";

export const screens = { Crash, MainWindow, SignIn };
