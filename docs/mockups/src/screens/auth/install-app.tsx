// Install the ciproxy GitHub App on an organization. GitHub owns the choice of
// organization and repositories; ciproxy shows the three permissions that the
// app asks for. A member who is not an owner asks the owners on GitHub; when
// one approves, the member who asked becomes the first Admin in ciproxy.

import { Check, Github } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";
import { appPermissions } from "../../fixtures/github-app.ts";
import { requester } from "../../fixtures/members.ts";
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
  const pending = state === "pending-approval";
  const login = pending ? requester.login : user.login;
  return (
    <AuthShell viewport={viewport} footer={`Signed in as ${login} on GitHub.`}>
      <div className="stack-tight">
        <h1 className="auth-title">Install ciproxy on GitHub</h1>
        <p className="auth-lead">
          Pick the organization and its repositories on GitHub. ciproxy then
          runs their jobs that use{" "}
          <span className="inline-code">runs-on: ciproxy</span>.
        </p>
      </div>
      {pending ? (
        <Notice tone="info" title={`Waiting for an owner of ${org.login}`}>
          GitHub sent your request to the owners of{" "}
          <span className="nowrap">{org.login}</span>. When one of them approves
          it, you become the first Admin of{" "}
          <span className="nowrap">{org.login}</span> in ciproxy, and setup
          continues here.
        </Notice>
      ) : (
        <ul className="permission-list small">
          {appPermissions.map((permission) => (
            <li key={permission.name}>
              <Check size={16} />
              <span>
                <strong>{permission.name}</strong> · {permission.access},{" "}
                {permission.use}
              </span>
            </li>
          ))}
        </ul>
      )}
      <div className="auth-actions">
        {pending ? (
          <>
            <LinkAction to={paths.onboarding} tone="primary" size="lg" block>
              Check again
            </LinkAction>
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
