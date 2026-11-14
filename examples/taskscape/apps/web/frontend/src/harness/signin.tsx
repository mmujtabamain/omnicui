// Mounts the sign-in page in its starting state: signed out, in the contract's mode.
import { useQuery } from "@tanstack/react-query";
import { AppHarness } from "./lib/AppHarness";
import { SignIn, type SignInMode } from "../screens/sign-in/SignIn";
import * as ep from "../api/endpoints";

/** The event hook the runner gives the harness: the first one present is used, once per success. */
interface EventHooks {
  emit?: (name: string, payload: { email: string }) => void;
  onSignedIn?: (payload: { email: string }) => void;
}

function fireSignedIn(hooks: EventHooks, email: string) {
  const payload = { email };
  if (hooks.emit) hooks.emit("signedIn", payload);
  else if (hooks.onSignedIn) hooks.onSignedIn(payload);
  else if ((window as any).omnicuiEmit) (window as any).omnicuiEmit("signedIn", payload);
  else window.dispatchEvent(new CustomEvent("omnicui:signedIn", { detail: payload }));
}

function SignInStage({ mode, hooks }: { mode: SignInMode; hooks: EventHooks }) {
  // The page is the signed-out screen: sign the demo account out before it is shown.
  const signedOut = useQuery({
    queryKey: ["harness", "signin", "signout"],
    queryFn: async () => {
      await ep.auth.signout().catch(() => null);
      return true;
    },
  });
  if (!signedOut.data) return null;
  return <SignIn mode={mode} onSignedIn={(email) => fireSignedIn(hooks, email)} />;
}

function SignInHarness(props: { mode?: string; dataset?: string } & EventHooks) {
  const mode: SignInMode = props.mode === "signup" ? "signup" : "signin";
  return (
    <AppHarness>
      <SignInStage mode={mode} hooks={props} />
    </AppHarness>
  );
}

export default { id: "app.signin", Component: SignInHarness, events: ["signedIn"] };
