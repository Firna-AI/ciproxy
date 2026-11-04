// The confirmation of Move to Free on the Plans page. The paid tier stays
// until the end of its period, then the organization is on Free, and jobs
// keep running on every machine, also past the limits of Free.

import { LinkAction } from "../../components/actions.tsx";
import { Dialog } from "../../components/dialog.tsx";
import { paths } from "../../components/paths.ts";
import { workspace } from "../../fixtures/org.ts";
import { ownMachines, startsText, tiers } from "../../fixtures/tiers.ts";
import { freeFrom, usageScenes } from "../../fixtures/usage.ts";

export function MoveToFreeDialog() {
  const usage = usageScenes["free-scheduled"];
  const { free, team } = tiers;
  return (
    <Dialog
      title={`Move ${workspace.name} to Free?`}
      footer={
        <>
          <LinkAction to={paths.plansOnTeam} tone="ghost">
            {`Keep ${team.name}`}
          </LinkAction>
          <LinkAction to={paths.freeScheduled} tone="primary">
            Move to Free
          </LinkAction>
        </>
      }
    >
      <p className="small">
        {team.name} stays until {usage.ends}, the end of this period. From{" "}
        {freeFrom}, <span className="nowrap">{workspace.name}</span> is on Free:{" "}
        {startsText(free.starts)} a month and {ownMachines(free.machines)}.
      </p>
      <p className="small">
        Jobs keep running on all {usage.machines} machines.
      </p>
    </Dialog>
  );
}
