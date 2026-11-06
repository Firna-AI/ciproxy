// The GitHub organizations of the workspace: a table on desktop and cards on
// mobile, each with its repository access, the way to its repositories, and
// Take out. The last organization of a workspace has no Take out, on every
// tier, as the answer D to item 51 says. An inactive organization shows why
// under its name, and the actions that clear its holds, as the answer D to
// item 83 says; the other rows do not change.

import { Avatar } from "@firna/ui/avatar";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import { DataTable, type Column } from "../../components/data-table.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import type { LinkedOrganization } from "../../fixtures/org-links.ts";
import { org, labsOrg } from "../../fixtures/org.ts";
import { HoldActions, InactiveStatus } from "./organization-hold.tsx";
import "./members.css";

const columns: readonly Column[] = [
  { key: "organization", label: "Organization", flex: 2 },
  { key: "access", label: "Repository access", flex: 2 },
  { key: "actions", label: "", width: 210, align: "right" },
];

/** The screens that exist: the repositories of mokly-ai, and Take out. */
const repositoriesPath: Readonly<Record<string, CataloguePath>> = {
  [org.login]: paths.repositories,
};
const takeOutPath: Readonly<Record<string, CataloguePath>> = {
  [labsOrg.login]: paths.takeOut,
};

function accessText({ organization, selected }: LinkedOrganization): string {
  const total = organization.repositories;
  return selected === undefined
    ? `All ${total} private repositories`
    : `${selected} of ${total} private repositories`;
}

function OrganizationName({ access }: { access: LinkedOrganization }) {
  const { organization, holds = [] } = access;
  const inactive = holds.length > 0;
  return (
    <span className={`person${inactive ? " person-top" : ""}`}>
      <Avatar
        label={organization.initial}
        accessibilityLabel={organization.login}
        size={28}
        shape="square"
      />
      <span className="person-text">
        <span className="strong">{organization.login}</span>
        <span className="caption">Since {organization.joined}</span>
        {inactive ? <InactiveStatus holds={holds} /> : null}
      </span>
    </span>
  );
}

function RowActions({
  access,
  only,
  turnOn,
}: {
  access: LinkedOrganization;
  only: boolean;
  turnOn?: CataloguePath;
}) {
  const login = access.organization.login;
  const repositories = repositoriesPath[login];
  const takeOut = takeOutPath[login];
  const holds = access.holds ?? [];
  return (
    <span className="org-actions-stack">
      {holds.length > 0 ? <HoldActions holds={holds} turnOn={turnOn} /> : null}
      <span className="row org-actions">
        {repositories ? (
          <LinkAction to={repositories} size="sm">
            Repositories
          </LinkAction>
        ) : (
          <Action size="sm">Repositories</Action>
        )}
        {only ? null : takeOut ? (
          <LinkAction to={takeOut} size="sm" tone="ghost">
            Take out
          </LinkAction>
        ) : (
          <Action size="sm" tone="ghost">
            Take out
          </Action>
        )}
      </span>
    </span>
  );
}

export function OrganizationList({
  viewport,
  list,
  turnOn,
}: {
  viewport: Viewport;
  list: readonly LinkedOrganization[];
  /** The Turn on dialog of an inactive organization, when it exists. */
  turnOn?: CataloguePath;
}) {
  const only = list.length === 1;
  if (viewport === "desktop") {
    return (
      <DataTable
        columns={columns}
        rows={list}
        rowKey={(access) => access.organization.login}
        cell={(access, key) => {
          if (key === "organization")
            return <OrganizationName access={access} />;
          if (key === "access")
            return <span className="small">{accessText(access)}</span>;
          return <RowActions access={access} only={only} turnOn={turnOn} />;
        }}
      />
    );
  }
  return (
    <ul className="member-cards">
      {list.map((access) => (
        <li key={access.organization.login} className="member-card">
          <OrganizationName access={access} />
          <span className="small">{accessText(access)}</span>
          <RowActions access={access} only={only} turnOn={turnOn} />
        </li>
      ))}
    </ul>
  );
}
