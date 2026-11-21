// The notices above the savings report: a machine cost that nobody set, a
// plan price or a registration date that ciproxy does not know, an
// alternative without a complete price, and a report that still refreshes.
// Only an Admin enters what a machine costs, so everyone else reads Ask an
// Admin in place of Add costs. The other gaps have no action.

import { LinkAction } from "../../components/actions.tsx";
import { Busy, Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";
import { AskAdmin } from "../../components/role-actions.tsx";
import type { Role } from "../../fixtures/members.ts";
import type {
  Alternative,
  ReportFigures,
} from "../../fixtures/savings-states.ts";

const names: Readonly<Record<Alternative, string>> = {
  blacksmith: "Blacksmith",
  github: "GitHub-hosted runners",
};

export function SavingsNotices({
  figures,
  role,
}: {
  figures: ReportFigures;
  role: Role;
}) {
  const machineGap = figures.ownCost === null || figures.ownPartial;
  const gap = figures.priceGap;
  return (
    <>
      {figures.updating ? (
        <p className="row small muted" role="status">
          <Busy />
          Updating the report…
        </p>
      ) : null}
      {machineGap ? (
        <Notice
          tone="warning"
          title="Add what your machines cost"
          actions={
            role === "admin" ? (
              <LinkAction to={paths.machine} size="sm">
                Add costs
              </LinkAction>
            ) : (
              <AskAdmin />
            )
          }
        >
          The report needs the monthly cost of each machine to show your
          savings.
        </Notice>
      ) : null}
      {figures.planPartial ? (
        <Notice tone="info" title="Some costs are not known">
          Some plan prices or machine registration dates are not known, so the
          report shows no savings. It still shows the costs that ciproxy knows.
        </Notice>
      ) : null}
      {gap ? (
        <Notice
          tone="info"
          title={`${names[gap.alternative]} has no price for one VM size`}
        >
          Its price list has no size with at least {gap.size}, so the report
          leaves out its cost and the savings figure.
        </Notice>
      ) : null}
    </>
  );
}
