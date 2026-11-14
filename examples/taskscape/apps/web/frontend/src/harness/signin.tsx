// Mounts the sign-in page in its starting state: signed out, in the contract's mode.
import { useQuery } from "@tanstack/react-query";
import { AppHarness } from "./lib/AppHarness";
import { SignIn, type SignInMode } from "../screens/sign-in/SignIn";
import * as ep from "../api/endpoints";

function SignInStage({ mode }: { mode: SignInMode }) {
  // The page is the signed-out screen: sign the demo account out before it is shown.
  const signedOut = useQuery({
    queryKey: ["harness", "signin", "signout"],
    queryFn: async () => {
      await ep.auth.signout().catch(() => null);
      return true;
    },
  });
  if (!signedOut.data) return null;
  return (
    <SignIn
      mode={mode}
      onSignedIn={(email) => {
        (window as any).omnicuiEvent?.("signedIn", { email });
      }}
    />
  );
}

function SignInHarness(props: { mode?: string; dataset?: string }) {
  const mode: SignInMode = props.mode === "signup" ? "signup" : "signin";
  return (
    <AppHarness>
      <SignInStage mode={mode} />
    </AppHarness>
  );
}

export default { id: "app.signin", Component: SignInHarness, events: ["signedIn"] };
