// The terms step: the page before each GitHub sign-in that does not start on
// sign-in. The return from an install on GitHub opens it, and so do the links
// that leave for a new GitHub sign-in, such as Check again of a sign-in with
// no workspace. It shows the terms notice of sign-in, and Continue with GitHub
// starts the sign-in. The page is static, so it names no install, no person,
// and no next page.

import { Github } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import { AuthShell } from "./auth-shell.tsx";
import { TermsNotice } from "./terms-notice.tsx";

export function TermsStepScreen({ viewport }: { viewport: Viewport }) {
  return (
    <AuthShell viewport={viewport} footer={<TermsNotice />}>
      <div className="stack-tight">
        <h1 className="auth-title">Sign in to continue</h1>
        <p className="auth-lead">
          ciproxy checks your access with your GitHub account.
        </p>
      </div>
      <LinkAction
        to={paths.onboarding}
        tone="primary"
        size="lg"
        icon={Github}
        block
      >
        Continue with GitHub
      </LinkAction>
    </AuthShell>
  );
}
