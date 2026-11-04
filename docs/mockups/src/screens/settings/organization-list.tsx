// The GitHub organizations of the workspace: a table on desktop and cards on
// mobile, each with its repository access, the way to its repositories, and
// Take out. The only organization of a workspace has no Take out.

import { Avatar } from "@firna/ui/avatar";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import { DataTable, type Column } from "../../components/data-table.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { org, labsOrg } from "../../fixtures/org.ts";
import type { OrganizationAccess } from "../../fixtures/settings.ts";
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

function accessText({ organization, selected }: OrganizationAccess): string {
  const total = organization.repositories;
  return selected === undefined
    ? `All ${total} private repositories`
    : `${selected} of ${total} private repositories`;
}

function OrganizationName({ access }: { access: OrganizationAccess }) {
  const { organization } = access;
  return (
    <span className="person">
      <Avatar
        label={organization.initial}
        accessibilityLabel={organization.login}
        size={28}
        shape="square"
      />
      <span className="person-text">
        <span className="strong">{organization.login}</span>
        <span className="caption">Since {organization.joined}</span>
      </span>
    </span>
  );
}

function RowActions({
  access,
  only,
}: {
  access: OrganizationAccess;
  only: boolean;
}) {
  const login = access.organization.login;
  const repositories = repositoriesPath[login];
  const takeOut = takeOutPath[login];
  return (
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
  );
}

export function OrganizationList({
  viewport,
  list,
}: {
  viewport: Viewport;
  list: readonly OrganizationAccess[];
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
          return <RowActions access={access} only={only} />;
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
          <RowActions access={access} only={only} />
        </li>
      ))}
    </ul>
  );
}
