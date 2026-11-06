// Compare the three tiers and choose one. The tier that covers the recent use
// carries a tag; the current tier cannot be chosen again. On Team, Choose
// Business reviews a move between paid tiers, and Move to Free asks first. A
// Billing person chooses as an Admin does. A Viewer compares the tiers and
// reads who can choose in place of each action. On Free in December, with an
// inactive organization, the page states the organizations and the machines
// of the workspace. The catalogue has no checkout of that day, so its choices
// open no screen.

import type { Viewport } from "@mokly/mokly";

import { PageHeader } from "../../components/page.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { canPay } from "../../components/role-actions.tsx";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { formatCount } from "../../fixtures/format.ts";
import type { Role } from "../../fixtures/members.ts";
import { decemberOrganizationCount } from "../../fixtures/org-links.ts";
import { organizationsText, type TierKey } from "../../fixtures/org.ts";
import { savings } from "../../fixtures/savings.ts";
import { ownMachines, tierOrder, tiers } from "../../fixtures/tiers.ts";
import { freePeriodEnd, usageScenes } from "../../fixtures/usage.ts";
import { MoveToFreeDialog } from "./move-to-free-dialog.tsx";
import { TierCard } from "./tier-card.tsx";
import "./billing.css";

export const plansStates = [
  "from-free",
  "on-team",
  "move-to-free",
  "inactive-organization",
] as const;
export type PlansState = (typeof plansStates)[number];

const freeUse = usageScenes["free-over"];
const teamUse = usageScenes.steady;
/** The first days of Free in December, with three machines. */
const decemberUse = usageScenes["free-organizations"];
const lastThirtyDays = savings.starts.own + savings.starts.boat;

type PlansContext = {
  current: TierKey;
  covers: TierKey;
  use: string;
  /** The Plan and usage screen that the breadcrumb opens. */
  usage: CataloguePath;
};

const onTeam: PlansContext = {
  current: "team",
  covers: "team",
  use: `Your last 30 days: ${formatCount(lastThirtyDays)} starts on ${ownMachines(teamUse.machines)}.`,
  usage: paths.billing,
};

const context: Record<PlansState, PlansContext> = {
  "from-free": {
    current: "free",
    covers: "team",
    use: `This month so far: ${formatCount(freeUse.starts)} starts on ${ownMachines(freeUse.machines)}. At this pace, about ${formatCount(freeUse.projected)} starts by ${freePeriodEnd}.`,
    usage: paths.usageFreeOver,
  },
  "on-team": onTeam,
  "move-to-free": onTeam,
  "inactive-organization": {
    current: "free",
    covers: "team",
    use: `Your workspace now: ${organizationsText(decemberOrganizationCount)} and ${ownMachines(decemberUse.machines)}. ${tiers.free.name} includes ${organizationsText(tiers.free.organizations ?? 1)}.`,
    usage: paths.freeOrganizations,
  },
};

/** The screen that a tier button opens, where the catalogue has one. */
function choicePath(
  state: PlansState,
  current: TierKey,
  tier: TierKey,
): CataloguePath | undefined {
  // The checkout of December has no screen, so its choices open none.
  if (state === "inactive-organization") return undefined;
  if (current === "free" && tier === "team") return paths.checkoutReview;
  if (current === "team" && tier === "business") return paths.moveToBusiness;
  if (current === "team" && tier === "free") return paths.moveToFree;
  return undefined;
}

export function PlansScreen({
  viewport,
  state,
  role = "admin",
}: {
  viewport: Viewport;
  state: PlansState;
  /** The role of the signed-in person, which decides the actions. */
  role?: Role;
}) {
  const { current, covers, use, usage } = context[state];
  return (
    <AppShell viewport={viewport} tier={current} active="billing" role={role}>
      <PageHeader
        crumbs={[{ label: "Plan and usage", to: usage }]}
        title="Plans"
        subtitle={use}
      />
      <div className="tier-grid">
        {tierOrder.map((key) => (
          <TierCard
            key={key}
            tier={tiers[key]}
            current={key === current}
            covers={key === covers && key !== current}
            to={choicePath(state, current, key)}
            canPay={canPay(role)}
          />
        ))}
      </div>
      <p className="caption">
        Limits are soft: jobs keep running past them, and ciproxy offers the
        plan or add-on that covers your use. Prices in US dollars, billed
        monthly through Stripe.
      </p>
      {state === "move-to-free" ? <MoveToFreeDialog /> : null}
    </AppShell>
  );
}
