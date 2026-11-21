// The account menu that the avatar of the top bar opens: the signed-in
// person, the role in the workspace, the docs, and Sign out. A person with
// the Billing or the Viewer role claims the Admin role here when ciproxy
// offers the claim. The menu reads only that offer, as the dashboard reads
// the claim of the session. ciproxy decides it from the person's own sign-in.

import { BookOpen, LogOut, ShieldCheck } from "lucide-react";
import { MockLink } from "@mokly/mokly";
import { Avatar } from "@firna/ui/avatar";

import { roles, type Role } from "../../fixtures/members.ts";
import { workspace } from "../../fixtures/org.ts";
import { Action } from "../actions.tsx";
import { paths } from "../paths.ts";
import { Tag } from "../status.tsx";
import type { ShellPerson } from "./app-shell.tsx";
import "./account-menu.css";

export function AccountMenu({
  person,
  role,
}: {
  person: ShellPerson;
  role: Role;
}) {
  const canClaim = person.canClaimAdmin === true;
  return (
    <div className="account-menu" role="menu" aria-label="Account">
      <div className="account-section account-person">
        <Avatar
          label={person.initials}
          accessibilityLabel={person.name}
          size={32}
          tone="soft"
        />
        <span className="stack-tight">
          <span className="strong">{person.name}</span>
          <span className="caption">{person.login}</span>
        </span>
      </div>
      <div className="account-section account-role">
        <span className="small muted">Role in {workspace.name}</span>
        <Tag>{roles[role].name}</Tag>
      </div>
      {canClaim ? (
        <div className="account-section">
          <p className="small">
            Your ownership on GitHub lets you claim the Admin role.
          </p>
          <Action tone="primary" size="sm" icon={ShieldCheck} block>
            Claim the Admin role
          </Action>
        </div>
      ) : null}
      <div className="account-section account-items">
        <MockLink to={paths.docsHome} className="account-item">
          <BookOpen size={16} strokeWidth={1.75} />
          <span>Docs</span>
        </MockLink>
        <MockLink to={paths.signIn} className="account-item">
          <LogOut size={16} strokeWidth={1.75} />
          <span>Sign out</span>
        </MockLink>
      </div>
    </div>
  );
}
