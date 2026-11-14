import { Component, useEffect, type ReactNode } from "react";
import { Navigate, Route, Routes, useNavigate } from "react-router-dom";
import { useQueryClient } from "@tanstack/react-query";
import { reportError, setUnauthorizedHandler } from "./api/client";
import { keys, useMe, useSettings } from "./api/queries";
import { useAppearance } from "./lib/appearance";
import { screens } from "./screens";

/** Signed out → the sign-in page; signed in → the app. */
export function App() {
  const me = useMe();
  const settings = useSettings();
  const navigate = useNavigate();
  const qc = useQueryClient();
  useAppearance(me.data ? settings.data : undefined);
  // A 401 while signed in means the session ended: back to the sign-in page. Signed out, a 401 is expected.
  useEffect(() => setUnauthorizedHandler(() => {
    if (!qc.getQueryData(keys.me)) return;
    qc.removeQueries({ queryKey: keys.settings });
    qc.invalidateQueries({ queryKey: keys.me });
    navigate("/signin");
  }), [qc, navigate]);

  // Only the very first check shows nothing; later re-checks keep the page on screen.
  if (me.isLoading) return null;
  const signedIn = !!me.data;
  return (
    <Crash>
      <Routes>
        <Route path="/signin" element={signedIn ? <Navigate to="/" replace /> : <screens.SignIn mode="signin" />} />
        <Route path="/signup" element={signedIn ? <Navigate to="/" replace /> : <screens.SignIn mode="signup" />} />
        <Route path="/*" element={signedIn ? <screens.MainWindow /> : <Navigate to="/signin" replace />} />
      </Routes>
    </Crash>
  );
}

/** A render crash is reported and shown as the crash screen instead of a blank page. */
class Crash extends Component<{ children: ReactNode }, { error: Error | null }> {
  state = { error: null as Error | null };
  static getDerivedStateFromError(error: Error) { return { error }; }
  componentDidCatch(error: Error, info: { componentStack?: string | null }) { reportError(`render crash: ${error.message}`, `${error.stack}\n${info.componentStack ?? ""}`); }
  render() { return this.state.error ? <screens.Crash error={this.state.error} /> : this.props.children; }
}
