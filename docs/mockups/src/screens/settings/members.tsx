// Members and roles: the people of the workspace in ciproxy and the role of
// each. An Admin gives each person a role, and a dialog names what the Admin
// role controls before an Admin gives it. A member of one of the GitHub
// organizations of the workspace with no role asks for access, and the
// request waits here, with the organization that the person belongs to.

import { Avatar } from "@firna/ui/avatar";
import { DropdownSelector } from "@firna/ui/dropdown";
import type { Viewport } from "@mokly/mokly";

import { Action, noop } from "../../components/actions.tsx";
import { DataTable, type Column } from "../../components/data-table.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { Tag } from "../../components/status.tsx";
import { dotJoin } from "../../fixtures/format.ts";
import {
  billingPerson,
  members,
  openRequests,
  requester,
  roleOrder,
  roles,
  type Member,
  type Role,
} from "../../fixtures/members.ts";
import { org, organizationList, user } from "../../fixtures/org.ts";
import { GrantAdminDialog } from "./grant-admin-dialog.tsx";
import { SettingsFrame } from "./settings-frame.tsx";
import "./members.css";

export const membersStates = ["members", "request", "grant-admin"] as const;
export type MembersState = (typeof membersStates)[number];

const roleOptions = roleOrder.map((role) => ({
  label: roles[role].name,
  value: role,
}));

const columns: readonly Column[] = [
  { key: "member", label: "Member", flex: 1 },
  { key: "role", label: "Role", width: 132 },
  { key: "since", label: "Since", width: 124 },
  { key: "remove", label: "", width: 96, align: "right" },
];

function RolePicker({ role, name }: { role: Role; name: string }) {
  return (
    <span className="role-picker">
      <DropdownSelector
        variant="field"
        size="sm"
        options={roleOptions}
        value={role}
        onValueChange={noop}
        triggerLabel={`Role of ${name}`}
      />
    </span>
  );
}

type Person = {
  name: string;
  login: string;
  initials: string;
  note?: string;
};

function PersonName({ person }: { person: Person }) {
  return (
    <span className="person">
      <Avatar
        label={person.initials}
        accessibilityLabel={person.name}
        size={28}
        tone="soft"
      />
      <span className="person-text">
        <span className="row">
          <span className="strong">{person.name}</span>
          {person.login === user.login ? <Tag>You</Tag> : null}
        </span>
        <span className="caption">
          {person.note ? dotJoin(person.login, person.note) : person.login}
        </span>
      </span>
    </span>
  );
}

function RemoveAction({ member }: { member: Member }) {
  return member.login === user.login ? null : (
    <Action size="sm" tone="ghost">
      Remove
    </Action>
  );
}

function cell(member: Member, key: string) {
  switch (key) {
    case "member":
      return <PersonName person={member} />;
    case "role":
      return <RolePicker role={member.role} name={member.name} />;
    case "since":
      return <span className="small muted num">{member.since}</span>;
    default:
      return <RemoveAction member={member} />;
  }
}

function MemberList({
  viewport,
  rows,
}: {
  viewport: Viewport;
  rows: readonly Member[];
}) {
  if (viewport === "desktop") {
    return (
      <DataTable
        columns={columns}
        rows={rows}
        rowKey={(member) => member.login}
        cell={cell}
      />
    );
  }
  return (
    <ul className="member-cards">
      {rows.map((member) => (
        <li key={member.login} className="member-card">
          <div className="spread">
            <PersonName person={member} />
            <RolePicker role={member.role} name={member.name} />
          </div>
          <div className="spread">
            <span className="caption">Since {member.since}</span>
            <RemoveAction member={member} />
          </div>
        </li>
      ))}
    </ul>
  );
}

function AccessRequest() {
  return (
    <Panel title="Asked for access">
      <div className="request-row">
        <span className="stack-tight">
          <PersonName person={requester} />
          <span className="caption">
            {dotJoin(`Asked on ${requester.asked}`, `member of ${org.login}`)}
          </span>
        </span>
        <div className="request-actions">
          <RolePicker role="viewer" name={requester.name} />
          <Action tone="primary" size="sm">
            Give access
          </Action>
          <Action tone="ghost" size="sm">
            Decline
          </Action>
        </div>
      </div>
    </Panel>
  );
}

/** The members, with the role that an Admin just picked for one of them. */
function withPick(person: Member, role: Role): readonly Member[] {
  return members.map((member) =>
    member.login === person.login ? { ...member, role } : member,
  );
}

export function MembersScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: MembersState;
}) {
  const granting = state === "grant-admin";
  return (
    <SettingsFrame
      viewport={viewport}
      active={paths.members}
      openRequests={state === "request" ? openRequests.length : 0}
      overlay={
        granting ? <GrantAdminDialog person={billingPerson} /> : undefined
      }
    >
      {state === "request" ? <AccessRequest /> : null}
      <Panel title="Members" flush>
        <MemberList
          viewport={viewport}
          rows={granting ? withPick(billingPerson, "admin") : members}
        />
      </Panel>
      <p className="caption">
        Other members of {organizationList("and")} on GitHub ask for access when
        they sign in.
      </p>
      <Panel title="Roles">
        <ul className="role-list">
          {roleOrder.map((role) => (
            <li key={role} className="role-row">
              <span className="strong role-name">{roles[role].name}</span>
              <span className="small muted">{roles[role].can}</span>
            </li>
          ))}
        </ul>
      </Panel>
    </SettingsFrame>
  );
}
