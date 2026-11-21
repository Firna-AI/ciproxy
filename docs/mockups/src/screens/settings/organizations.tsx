// The Organizations tab of Settings: the name of the workspace and its GitHub
// organizations. The workspace owns the tier, the members, the machines, the
// labels, the settings, and the Boat account, and a machine runs the jobs of
// every organization, as the answer to item 48 says. An Admin renames the
// workspace, adds an organization, which installs the GitHub App on it on
// GitHub, where GitHub decides who may finish the install, and takes one out.
// Free includes one organization, and Team and Business several. On Free, Add
// an organization stays: the new one joins inactive until Team, as the answer
// D to item 83 says. A workspace that keeps more after a move to Free shows
// the banner of that soft limit. The last linked organization has no Take
// out. An organization whose owner removed the App on GitHub stays in the tab
// for 30 days, as removed, with the day before which a new install keeps its
// job history.

import { Plus } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { organizationsText, workspace } from "../../fixtures/org.ts";
import { tiers } from "../../fixtures/tiers.ts";
import { OrganizationList } from "./organization-list.tsx";
import {
  organizationsScene,
  type OrganizationsState,
} from "./organization-scenes.tsx";
import { SettingsFrame } from "./settings-frame.tsx";

function FreeNotice() {
  const { free, team, business } = tiers;
  return (
    <Notice
      tone="info"
      title={`Run more organizations on ${team.name}`}
      actions={
        <LinkAction to={paths.plans} size="sm">
          Compare plans
        </LinkAction>
      }
    >
      {free.name} includes {organizationsText(free.organizations ?? 1)}.{" "}
      {team.name} and {business.name} run several, on the same machines.
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

export function OrganizationsScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: OrganizationsState;
}) {
  const scene = organizationsScene(state);
  const { free, team } = tiers;
  return (
    <SettingsFrame
      viewport={viewport}
      active={paths.organizations}
      tier={scene.tier}
      banner={scene.banner}
      overlay={scene.overlay}
    >
      <WorkspacePanel />
      {state === "free" ? <FreeNotice /> : null}
      <Panel
        title="Organizations"
        flush
        action={
          <Action size="sm" icon={Plus}>
            Add an organization
          </Action>
        }
      >
        <OrganizationList
          viewport={viewport}
          list={scene.list}
          turnOn={scene.turnOn}
        />
      </Panel>
      <p className="caption">
        Add an organization opens GitHub, where you pick the organization and
        its repositories.
        {scene.tier === "free"
          ? ` On ${free.name}, it stays inactive until ${team.name}.`
          : null}
      </p>
    </SettingsFrame>
  );
}
