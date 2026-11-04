// The page that a person with no role in ciproxy sees on every page of the
// workspace. A member of one of its GitHub organizations asked the Admins for
// access by signing in; the page names them, and an Admin gives the role in
// Settings, under Members. An owner of any of its organizations can claim the
// Admin role at once. A person who left every organization of the workspace
// lost the role at the membership check of each hour, as design call 35 says.

import { Avatar } from "@firna/ui/avatar";
import type { Viewport } from "@mokly/mokly";

import { Action } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { leaver } from "../../fixtures/access.ts";
import { members, owner, requester } from "../../fixtures/members.ts";
import { org, organizationList, workspace } from "../../fixtures/org.ts";
import { AuthShell } from "./auth-shell.tsx";

export const noRoleStates = ["member", "owner", "left"] as const;
export type NoRoleState = (typeof noRoleStates)[number];

const admins = members.filter((member) => member.role === "admin");

const people = { member: requester, owner, left: leaver } as const;

function AdminList() {
  return (
    <div className="stack-tight">
      <p className="caption">Admins of {workspace.name}</p>
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
  );
}

/** The person left the GitHub organization, so the role ended. */
function LeftBody() {
  return (
    <>
      <div className="stack-tight">
        <h1 className="auth-title">Your access to {workspace.name} ended</h1>
        <p className="auth-lead">
          GitHub no longer lists you in {organizationList("or")}, the
          organizations of this workspace. An owner of one of them can add you
          again on GitHub.
        </p>
      </div>
      <div className="auth-actions">
        <Action tone="primary" size="lg" block>
          Check again
        </Action>
        <Action tone="secondary" size="lg" block>
          Install on another organization
        </Action>
      </div>
    </>
  );
}

function WaitingBody({ state }: { state: "member" | "owner" }) {
  return (
    <>
      <div className="stack-tight">
        <h1 className="auth-title">Waiting for an Admin of {workspace.name}</h1>
        <p className="auth-lead">
          ciproxy sent your request for access to the Admins. When one of them
          gives you a role, <span className="nowrap">{workspace.name}</span>{" "}
          opens here.
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
      <AdminList />
      <div className="auth-actions">
        <Action tone="primary" size="lg" block>
          {state === "owner" ? "Claim the Admin role" : "Check again"}
        </Action>
        <Action tone="secondary" size="lg" block>
          {state === "owner"
            ? "Check again"
            : "Install on another organization"}
        </Action>
      </div>
    </>
  );
}

export function NoRoleScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: NoRoleState;
}) {
  return (
    <AuthShell
      viewport={viewport}
      footer={`Signed in as ${people[state].login} on GitHub.`}
    >
      {state === "left" ? <LeftBody /> : <WaitingBody state={state} />}
    </AuthShell>
  );
}
