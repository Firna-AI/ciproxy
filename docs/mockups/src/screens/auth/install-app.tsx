// Install the ciproxy GitHub App on an organization. GitHub owns the choice of
// organization and repositories; ciproxy shows what the app asks for.

import { Check, Github } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";
import { org, user } from "../../fixtures/org.ts";
import { AuthShell } from "./auth-shell.tsx";

export type InstallState = "install" | "pending-approval";

export function InstallAppScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: InstallState;
}) {
  return (
    <AuthShell
      viewport={viewport}
      footer={`Signed in as ${user.login} on GitHub.`}
    >
      <div className="stack-tight">
        <h1 className="auth-title">Install ciproxy on GitHub</h1>
        <p className="auth-lead">
          Pick the organization and its repositories on GitHub. ciproxy then
          runs their jobs that use{" "}
          <span className="inline-code">runs-on: ciproxy</span>.
        </p>
      </div>
      {state === "pending-approval" ? (
        <Notice tone="info" title={`Waiting for an owner of ${org.login}`}>
          GitHub sent your request to the owners of {org.login}. Setup continues
          here when one of them approves.
        </Notice>
      ) : (
        <ul className="permission-list small">
          <li>
            <Check size={16} />
            <span>
              <strong>Self-hosted runners</strong> · read and write, to start a
              runner for each job
            </span>
          </li>
          <li>
            <Check size={16} />
            <span>
              <strong>Metadata</strong> · read, to list your repositories
            </span>
          </li>
        </ul>
      )}
      <div className="auth-actions">
        {state === "pending-approval" ? (
          <>
            <Action tone="primary" size="lg" block>
              Check again
            </Action>
            <Action tone="secondary" size="lg" block>
              Install on another organization
            </Action>
          </>
        ) : (
          <LinkAction
            to={paths.onboarding}
            tone="primary"
            size="lg"
            icon={Github}
            block
          >
            Install on GitHub
          </LinkAction>
        )}
      </div>
    </AuthShell>
  );
}
