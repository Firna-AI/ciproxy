// Sign-in with GitHub, and the return after GitHub sign-in did not finish.

import { Github } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";
import { AuthShell } from "./auth-shell.tsx";

export type SignInState = "ready" | "failed";

export function SignInScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: SignInState;
}) {
  return (
    <AuthShell
      viewport={viewport}
      brandTo={paths.landing}
      footer="By signing in, you agree to the ciproxy Terms and Privacy policy."
    >
      <div className="stack-tight">
        <h1 className="auth-title">Run GitHub Actions on your own machines</h1>
        <p className="auth-lead">
          ciproxy starts each job on your machines first, and on Boat when they
          are busy.
        </p>
      </div>
      {state === "failed" ? (
        <Notice tone="danger" title="GitHub sign-in did not finish">
          Sign in with GitHub again to continue.
        </Notice>
      ) : null}
      <LinkAction
        to={paths.installApp}
        tone="primary"
        size="lg"
        icon={Github}
        block
      >
        Sign in with GitHub
      </LinkAction>
    </AuthShell>
  );
}
