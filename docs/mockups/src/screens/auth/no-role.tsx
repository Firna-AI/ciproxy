// The page that a person with no role in ciproxy sees on every page of the
// workspace. A member of one of its GitHub organizations asked the Admins for
// access by signing in; the page names them, and an Admin gives the role in
// Settings, under Members. When ciproxy offers the person the Admin claim,
// the page offers Claim the Admin role. The page reads only that offer, as
// the dashboard reads the claim of the session. A person who left every
// organization of the workspace lost the role at the membership check of
// each hour.

import { Avatar } from "@firna/ui/avatar";
import type { Viewport } from "@mokly/mokly";

import { Action } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { leaver } from "../../fixtures/access.ts";
import {
  members,
  owner,
  requester,
  type Member,
} from "../../fixtures/members.ts";
import { organizationList, workspace } from "../../fixtures/org.ts";
import { AuthShell } from "./auth-shell.tsx";

export const noRoleStates = ["member", "owner", "left"] as const;
export type NoRoleState = (typeof noRoleStates)[number];

const admins = members.filter((member) => member.role === "admin");

/** The person of a state, and whether ciproxy offers the Admin claim. */
type NoRolePerson = Pick<
  Member,
  "name" | "login" | "initials" | "canClaimAdmin"
>;

const people: Readonly<Record<NoRoleState, NoRolePerson>> = {
  member: requester,
  owner,
  left: leaver,
};

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

function WaitingBody({ claim }: { claim: boolean }) {
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
      {claim ? (
        <Notice tone="info" title="You can claim the Admin role">
          Your ownership on GitHub lets you claim it without waiting for an
          Admin.
        </Notice>
      ) : null}
      <AdminList />
      <div className="auth-actions">
        <Action tone="primary" size="lg" block>
          {claim ? "Claim the Admin role" : "Check again"}
        </Action>
        <Action tone="secondary" size="lg" block>
          {claim ? "Check again" : "Install on another organization"}
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
      {state === "left" ? (
        <LeftBody />
      ) : (
        <WaitingBody claim={people[state].canClaimAdmin === true} />
      )}
    </AuthShell>
  );
}
