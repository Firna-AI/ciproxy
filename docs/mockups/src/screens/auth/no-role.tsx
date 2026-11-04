// The page that a member of the GitHub organization with no role in ciproxy
// sees. The sign-in asked the Admins for access; the page names them, and an
// Admin gives the role in Settings, under Members. An owner of the GitHub
// organization can claim the Admin role at once.

import { Avatar } from "@firna/ui/avatar";
import type { Viewport } from "@mokly/mokly";

import { Action } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { members, owner, requester } from "../../fixtures/members.ts";
import { org } from "../../fixtures/org.ts";
import { AuthShell } from "./auth-shell.tsx";

export const noRoleStates = ["member", "owner"] as const;
export type NoRoleState = (typeof noRoleStates)[number];

const admins = members.filter((member) => member.role === "admin");

export function NoRoleScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: NoRoleState;
}) {
  const person = state === "owner" ? owner : requester;
  return (
    <AuthShell
      viewport={viewport}
      footer={`Signed in as ${person.login} on GitHub.`}
    >
      <div className="stack-tight">
        <h1 className="auth-title">Waiting for an Admin of {org.login}</h1>
        <p className="auth-lead">
          ciproxy sent your request for access to the Admins. When one of them
          gives you a role, <span className="nowrap">{org.login}</span> opens
          here.
        </p>
      </div>
      {state === "owner" ? (
        <Notice
          tone="info"
          title={`You are an owner of ${org.login} on GitHub`}
        >
          An owner can claim the Admin role at once.
        </Notice>
      ) : null}
      <div className="stack-tight">
        <p className="caption">Admins of {org.login}</p>
        <ul className="admin-list small">
          {admins.map((admin) => (
            <li key={admin.login}>
              <Avatar
                label={admin.initials}
                accessibilityLabel={admin.name}
                size={24}
                tone="soft"
              />
              <span>
                <strong>{admin.name}</strong>{" "}
                <span className="muted">{admin.login}</span>
              </span>
            </li>
          ))}
        </ul>
      </div>
      <div className="auth-actions">
        {state === "owner" ? (
          <>
            <Action tone="primary" size="lg" block>
              Claim the Admin role
            </Action>
            <Action tone="secondary" size="lg" block>
              Check again
            </Action>
          </>
        ) : (
          <>
            <Action tone="primary" size="lg" block>
              Check again
            </Action>
            <Action tone="secondary" size="lg" block>
              Install on another organization
            </Action>
          </>
        )}
      </div>
    </AuthShell>
  );
}
