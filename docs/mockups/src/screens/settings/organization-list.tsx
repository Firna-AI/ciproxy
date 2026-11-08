// The GitHub organizations of the workspace: a table on desktop and cards on
// mobile. A linked organization shows its repository access, the way to its
// repositories, and Take out. The last linked organization of a workspace has
// no Take out, on every tier, as the answer D to item 51 says; a removed one
// does not count. An inactive organization shows why under its name, and the
// actions that clear its holds, as the answer D to item 83 says. A removed
// organization shows the day before which a new install keeps its job
// history, and Install again; ciproxy has no GitHub access left to it.

import { Avatar } from "@firna/ui/avatar";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import { DataTable, type Column } from "../../components/data-table.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { DotLabel } from "../../components/status.tsx";
import {
  canTakeOut,
  type LinkedOrganization,
  type RemovedOrganization,
  type WorkspaceOrganization,
} from "../../fixtures/org-links.ts";
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

function accessText(row: WorkspaceOrganization): string {
  if (row.state === "removed") return "No repository access";
  const total = row.organization.repositories;
  return row.selected === undefined
    ? `All ${total} private repositories`
    : `${row.selected} of ${total} private repositories`;
}

function RemovedStatus({ row }: { row: RemovedOrganization }) {
  return (
    <span className="org-status">
      <DotLabel tone="idle">App removed</DotLabel>
      <span className="caption">
        Install it again before {row.installBefore} to keep its job history
      </span>
    </span>
  );
}

function OrganizationName({ row }: { row: WorkspaceOrganization }) {
  const { organization } = row;
  const holds = row.state === "linked" ? (row.holds ?? []) : [];
  const status = row.state === "removed" || holds.length > 0;
  return (
    <span className={`person${status ? " person-top" : ""}`}>
      <Avatar
        label={organization.initial}
        accessibilityLabel={organization.login}
        size={28}
        shape="square"
      />
      <span className="person-text">
        <span className="strong">{organization.login}</span>
        {row.state === "removed" ? (
          <RemovedStatus row={row} />
        ) : (
          <>
            <span className="caption">Since {organization.joined}</span>
            {holds.length > 0 ? <InactiveStatus holds={holds} /> : null}
          </>
        )}
      </span>
    </span>
  );
}

function LinkedActions({
  row,
  takeOut,
  turnOn,
}: {
  row: LinkedOrganization;
  takeOut: boolean;
  turnOn?: CataloguePath;
}) {
  const login = row.organization.login;
  const repositories = repositoriesPath[login];
  const takeOutTo = takeOutPath[login];
  const holds = row.holds ?? [];
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
        {!takeOut ? null : takeOutTo ? (
          <LinkAction to={takeOutTo} size="sm" tone="ghost">
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

/** Install again starts Add an organization; the return links it at once. */
function RemovedActions() {
  return (
    <span className="row org-actions">
      <LinkAction to={paths.organizations} size="sm">
        Install again
      </LinkAction>
    </span>
  );
}

function RowActions({
  row,
  list,
  turnOn,
}: {
  row: WorkspaceOrganization;
  list: readonly WorkspaceOrganization[];
  turnOn?: CataloguePath;
}) {
  return row.state === "removed" ? (
    <RemovedActions />
  ) : (
    <LinkedActions row={row} takeOut={canTakeOut(row, list)} turnOn={turnOn} />
  );
}

export function OrganizationList({
  viewport,
  list,
  turnOn,
}: {
  viewport: Viewport;
  list: readonly WorkspaceOrganization[];
  /** The Turn on dialog of an inactive organization, when it exists. */
  turnOn?: CataloguePath;
}) {
  if (viewport === "desktop") {
    return (
      <DataTable
        columns={columns}
        rows={list}
        rowKey={(row) => row.organization.login}
        cell={(row, key) => {
          if (key === "organization") return <OrganizationName row={row} />;
          if (key === "access")
            return <span className="small">{accessText(row)}</span>;
          return <RowActions row={row} list={list} turnOn={turnOn} />;
        }}
      />
    );
  }
  return (
    <ul className="member-cards">
      {list.map((row) => (
        <li key={row.organization.login} className="member-card">
          <OrganizationName row={row} />
          <span className="small">{accessText(row)}</span>
          <RowActions row={row} list={list} turnOn={turnOn} />
        </li>
      ))}
    </ul>
  );
}
