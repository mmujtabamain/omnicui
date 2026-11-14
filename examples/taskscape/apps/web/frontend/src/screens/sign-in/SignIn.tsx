// The signed-out account page: sign in, or create an account. One page in two modes.
import * as React from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Button } from "@taskscape/ui/button";
import { Card } from "@taskscape/ui/card";
import { Heading } from "@taskscape/ui/heading";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { keys, useWrite } from "../../api/queries";
import { FormField } from "./FormField";

export type SignInMode = "signin" | "signup";

interface Problems {
  form: string;
  name: string;
  email: string;
  password: string;
}

const NONE: Problems = { form: "", name: "", email: "", password: "" };

interface SubmitRequest {
  mode: SignInMode;
  name: string;
  email: string;
  password: string;
}

/** The refusal's JSON body, when the error carries one. */
function bodyOf(err: unknown): Record<string, unknown> {
  const e = err as { body?: unknown; data?: unknown; message?: unknown } | null;
  for (const candidate of [e?.body, e?.data]) {
    if (candidate && typeof candidate === "object") return candidate as Record<string, unknown>;
  }
  if (typeof e?.message === "string") {
    const start = e.message.indexOf("{");
    if (start >= 0) {
      try {
        const parsed: unknown = JSON.parse(e.message.slice(start));
        if (parsed && typeof parsed === "object") return parsed as Record<string, unknown>;
      } catch {
        // Not JSON: fall through to the generic message.
      }
    }
  }
  return {};
}

/** The first message of a DRF-style value: a string, or a list of strings. */
function first(value: unknown): string {
  if (Array.isArray(value)) return first(value[0]);
  return typeof value === "string" ? value : "";
}

/** Maps a refused request onto the fields it names; the rest goes under the form. */
function problemsOf(err: unknown, mode: SignInMode): Problems {
  const body = bodyOf(err);
  const name = first(body.name);
  const email = first(body.email);
  const password = first(body.password);
  // The sign-in form has no name field: every server message is shown under the form.
  if (mode === "signin") {
    const message = first(body.detail) || first(body.non_field_errors) || email || password;
    return { ...NONE, form: message || "Check your email and password, then try again." };
  }
  const detail = first(body.detail) || first(body.non_field_errors);
  const fieldRefused = name !== "" || email !== "" || password !== "";
  return {
    form: fieldRefused ? detail : detail || "We could not create the account. Try again.",
    name,
    email,
    password,
  };
}

export function SignIn({
  mode: initialMode = "signin",
  onSignedIn,
}: {
  mode?: SignInMode;
  /** Called once a sign-in or create-account request succeeds, before the main window opens. */
  onSignedIn?: (email: string) => void;
}) {
  const [mode, setMode] = React.useState<SignInMode>(initialMode);
  const [name, setName] = React.useState("");
  const [email, setEmail] = React.useState("");
  const [password, setPassword] = React.useState("");
  const [problems, setProblems] = React.useState<Problems>(NONE);
  const queryClient = useQueryClient();

  React.useEffect(() => {
    setMode(initialMode);
    setProblems(NONE);
  }, [initialMode]);

  const write = useWrite((req: SubmitRequest) =>
    req.mode === "signin"
      ? ep.auth.signin(req.email, req.password)
      : ep.auth.signup(req.email, req.password, req.name || undefined),
  );
  const busy = write.isPending;
  const isSignin = mode === "signin";

  const submit = () => {
    if (busy) return;
    const req: SubmitRequest = { mode, name: name.trim(), email: email.trim(), password };
    setProblems(NONE);
    write.mutate([req], {
      onSuccess: (me) => {
        onSignedIn?.(req.email);
        queryClient.setQueryData(keys.me, me);
      },
      onError: (err) => setProblems(problemsOf(err, mode)),
    });
  };

  const switchMode = () => {
    setMode(isSignin ? "signup" : "signin");
    setProblems(NONE);
  };

  return (
    <div className="flex h-full min-h-0 items-center justify-center overflow-auto bg-surface-1 p-space-5">
      <Card data-part="app.signin.card" className="w-[22rem] max-w-full">
        <div className="flex flex-col gap-space-5 p-space-6">
          <div className="flex flex-col items-center gap-space-3">
            {isSignin ? (
              <Heading as="h1" size="md" data-part="app.signin.appName" className="text-center">
                Taskscape
              </Heading>
            ) : (
              <Heading as="h1" size="md" className="text-center">
                Create your account
              </Heading>
            )}
            <Text as="p" size="sm" tone="secondary" align="center">
              {isSignin ? "Sign in to your tasks" : "Start keeping your tasks in one place"}
            </Text>
          </div>

          <div className="flex flex-col gap-space-4">
            {!isSignin && (
              <FormField
                label="Name (optional)"
                ariaLabel="Name"
                part="app.signin.nameField"
                errorPart="app.signin.nameError"
                value={name}
                onChange={setName}
                onSubmit={submit}
                error={problems.name}
              />
            )}
            <FormField
              label="Email"
              ariaLabel="Email"
              part="app.signin.emailField"
              errorPart="app.signin.emailError"
              value={email}
              onChange={setEmail}
              onSubmit={submit}
              error={problems.email}
              spellCheck={false}
            />
            <FormField
              label="Password"
              ariaLabel="Password"
              part="app.signin.passwordField"
              errorPart="app.signin.passwordError"
              value={password}
              onChange={setPassword}
              onSubmit={submit}
              error={problems.password}
              masked
              spellCheck={false}
            />
          </div>

          <div className="flex flex-col gap-space-4">
            <Button
              type="button"
              data-part="app.signin.submitButton"
              variant="primary"
              size="xl"
              label={isSignin ? "Sign in" : "Create account"}
              busy={busy}
              onClick={submit}
              className="h-[2.5rem]"
            />
            {problems.form !== "" && (
              <Text as="p" role="status" data-part="app.signin.formError" size="sm" tone="danger" align="center">
                {problems.form}
              </Text>
            )}
          </div>

          <div className="flex justify-center">
            <Button
              type="button"
              data-part="app.signin.modeLink"
              variant="link"
              size="md"
              tone="accent"
              width="content"
              label={isSignin ? "New here? Create an account" : "Already have an account? Sign in"}
              onClick={switchMode}
            />
          </div>
        </div>
      </Card>
    </div>
  );
}
