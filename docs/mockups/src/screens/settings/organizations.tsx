// The Organizations tab of Settings: the GitHub organizations of the
// workspace. The workspace owns the tier, the members, the machines, the
// labels, the settings, and the Boat account, and a machine runs the jobs of
// every organization, as the answer to item 48 says. An Admin adds an
// organization, which installs the GitHub App on it on GitHub, where GitHub
// decides who may finish the install, and takes one out. Free holds one
// organization; Team and Business hold several.

import { Plus } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Dialog } from "../../components/dialog.tsx";
import { Notice } from "../../components/feedback.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { labsOrg } from "../../fixtures/org.ts";
import { organizationAccess } from "../../fixtures/settings.ts";
import { tiers } from "../../fixtures/tiers.ts";
import { OrganizationList } from "./organization-list.tsx";
import { SettingsFrame } from "./settings-frame.tsx";

export const organizationsStates = ["team", "free", "take-out"] as const;
export type OrganizationsState = (typeof organizationsStates)[number];

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

function TakeOutDialog() {
  return (
    <Dialog
      title={`Take ${labsOrg.login} out of this workspace?`}
      footer={
        <>
          <LinkAction to={paths.organizations} tone="ghost">
            Cancel
          </LinkAction>
          <LinkAction to={paths.organizations} tone="danger">
            Take out
          </LinkAction>
        </>
      }
    >
      <p className="small">
        No new job of <span className="nowrap">{labsOrg.login}</span> starts on
        ciproxy, and its running jobs finish. Its jobs stay in the job history,
        and the GitHub App stays installed on it until an owner removes it on
        GitHub.
      </p>
    </Dialog>
  );
}

export function OrganizationsScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: OrganizationsState;
}) {
  const free = state === "free";
  const list = free ? organizationAccess.slice(0, 1) : organizationAccess;
  return (
    <SettingsFrame
      viewport={viewport}
      active={paths.organizations}
      tier={free ? "free" : "team"}
      overlay={state === "take-out" ? <TakeOutDialog /> : undefined}
    >
      <p className="small muted settings-intro">
        The organizations of this workspace share its plan, members, machines,
        labels, and Boat account. Each machine runs the jobs of every
        organization.
      </p>
      {free ? <FreeNotice /> : null}
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
