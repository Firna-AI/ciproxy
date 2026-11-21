// The page after a person installs the ciproxy GitHub App on a personal
// account. ciproxy serves organizations only, as design call 36 says: the
// page says so, offers the install on an organization, which GitHub gives for
// free, and links to the removal of the App from the personal account.

import { ExternalLink, Github } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action } from "../../components/actions.tsx";
import { user } from "../../fixtures/org.ts";
import { AuthShell } from "./auth-shell.tsx";

export function PersonalAccountScreen({ viewport }: { viewport: Viewport }) {
  return (
    <AuthShell
      viewport={viewport}
      footer={`Signed in as ${user.login} on GitHub.`}
    >
      <div className="stack-tight">
        <h1 className="auth-title">ciproxy works with GitHub organizations</h1>
        <p className="auth-lead">
          You installed it on <span className="nowrap">{user.login}</span>, your
          personal account. Install it on an organization instead. A GitHub
          organization is free.
        </p>
      </div>
      <div className="auth-actions">
        <Action tone="primary" size="lg" icon={Github} block>
          Install on an organization
        </Action>
        <Action tone="secondary" size="lg" icon={ExternalLink} block>
          {`Remove from ${user.login}`}
        </Action>
      </div>
    </AuthShell>
  );
}
