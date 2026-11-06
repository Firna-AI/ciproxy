// The Organizations tab of Settings: the name of the workspace and its GitHub
// organizations. The workspace owns the tier, the members, the machines, the
// labels, the settings, and the Boat account, and a machine runs the jobs of
// every organization, as the answer to item 48 says. An Admin renames the
// workspace, adds an organization, which installs the GitHub App on it on
// GitHub, where GitHub decides who may finish the install, and takes one out.
// Free holds one organization; Team and Business hold several. A workspace
// that keeps more after a move to Free shows the banner of that soft limit.
// The last organization has no Take out, on every tier.

import type { ReactNode } from "react";
import { Plus } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { usageBanner } from "../../components/usage-banner.tsx";
import { workspace } from "../../fixtures/org.ts";
import { organizationAccess } from "../../fixtures/settings.ts";
import { tiers } from "../../fixtures/tiers.ts";
import {
  RenameWorkspaceDialog,
  TakeOutDialog,
} from "./organization-dialogs.tsx";
import { OrganizationList } from "./organization-list.tsx";
import { SettingsFrame } from "./settings-frame.tsx";

export const organizationsStates = [
  "team",
  "free",
  "free-over",
  "one-on-team",
  "take-out",
  "rename",
] as const;
export type OrganizationsState = (typeof organizationsStates)[number];

/** The states on Free, and the states with one organization. */
const onFree: readonly OrganizationsState[] = ["free", "free-over"];
const single: readonly OrganizationsState[] = ["free", "one-on-team"];

/** "one organization", "3 organizations". */
function organizationsText(count: number): string {
  return count === 1 ? "one organization" : `${count} organizations`;
}

function FreeNotice() {
  const { free, team, business } = tiers;
  return (
    <Notice
      tone="info"
      title={`Add organizations on ${team.name}`}
      actions={
        <LinkAction to={paths.plans} size="sm">
          Compare plans
        </LinkAction>
      }
    >
      {free.name} holds {organizationsText(free.organizations ?? 1)}.{" "}
      {team.name} and {business.name} hold several, on the same machines.
    </Notice>
  );
}

/** The name of the workspace, which a new workspace takes from GitHub. */
function WorkspacePanel() {
  return (
    <Panel title="Workspace">
      <div className="spread spread-stack">
        <div className="stack-tight settings-intro">
          <p className="strong">{workspace.name}</p>
          <p className="caption">
            Its organizations share its plan, members, machines, labels, and
            Boat account. Each machine runs the jobs of every organization.
          </p>
        </div>
        <LinkAction to={paths.renameWorkspace} size="sm">
          Rename
        </LinkAction>
      </div>
    </Panel>
  );
}

function overlay(state: OrganizationsState): ReactNode {
  if (state === "take-out") return <TakeOutDialog />;
  return state === "rename" ? <RenameWorkspaceDialog /> : undefined;
}

export function OrganizationsScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: OrganizationsState;
}) {
  const free = onFree.includes(state);
  const list = single.includes(state)
    ? organizationAccess.slice(0, 1)
    : organizationAccess;
  return (
    <SettingsFrame
      viewport={viewport}
      active={paths.organizations}
      tier={free ? "free" : "team"}
      banner={
        state === "free-over"
          ? usageBanner("free-organizations", "other")
          : undefined
      }
      overlay={overlay(state)}
    >
      <WorkspacePanel />
      {state === "free" ? <FreeNotice /> : null}
      <Panel
        title="Organizations"
        flush
        action={
          free ? undefined : (
            <Action size="sm" icon={Plus}>
              Add an organization
            </Action>
          )
        }
      >
        <OrganizationList viewport={viewport} list={list} />
      </Panel>
      {free ? null : (
        <p className="caption">
          Add an organization opens GitHub, where you pick the organization and
          its repositories.
        </p>
      )}
    </SettingsFrame>
  );
}
