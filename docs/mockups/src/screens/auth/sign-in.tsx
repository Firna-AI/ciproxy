// Sign-in with GitHub, the return after GitHub sign-in did not finish, and
// the page after a sign-in ended. A sign-in lasts 30 days, as design call 35
// says, and the same button signs in again.

import { Github } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction, TextLink } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";
import { signIn } from "../../fixtures/access.ts";
import { unbroken } from "../../fixtures/format.ts";
import { AuthShell } from "./auth-shell.tsx";

export type SignInState = "ready" | "failed" | "expired";

function StateNotice({ state }: { state: SignInState }) {
  if (state === "failed") {
    return (
      <Notice tone="danger" title="GitHub sign-in did not finish">
        Sign in with GitHub again to continue.
      </Notice>
    );
  }
  if (state === "expired") {
    return (
      <Notice
        tone="info"
        title={`Your sign-in ended after ${unbroken(`${signIn.days} days`)}`}
      >
        Sign in with GitHub again to continue.
      </Notice>
    );
  }
  return null;
}

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
      footer={
        <>
          By signing in, you agree to the ciproxy{" "}
          <TextLink to={paths.terms}>Terms</TextLink> and{" "}
          <TextLink to={paths.privacy}>Privacy policy</TextLink>.
        </>
      }
    >
      <div className="stack-tight">
        <h1 className="auth-title">Run GitHub Actions on your own machines</h1>
        <p className="auth-lead">
          ciproxy starts each job on your machines first, and on Boat when they
          are busy.
        </p>
      </div>
      <StateNotice state={state} />
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
