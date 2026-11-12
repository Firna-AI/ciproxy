// Compare the three tiers and choose one. The tier that covers the recent use
// carries a tag; the current tier cannot be chosen again. On Team, Choose
// Business reviews a move between paid tiers, and Move to Free asks first. A
// Billing person chooses as an Admin does. A Viewer compares the tiers and
// reads who can choose in place of each action. On Free in December, with an
// inactive organization, the page states the organizations and the machines
// of the workspace. The catalogue has no checkout of that day, so its choices
// open no screen. As the current API gives it, while a payment waits on Free
// in September, the page says so, with Continue payment when this tab saved
// the Stripe page, and no tier can be chosen until the payment ends. The API
// gives no add-on prices, so those cards name none. When the page did not
// get the result of Move to Free, its dialog says so.

import type { Viewport } from "@mokly/mokly";

import { apiBanner } from "../../components/api-banner.tsx";
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
import { PendingNotice } from "./api-notices.tsx";
import { inactivePages, inactiveUse } from "./billing-scenes.tsx";
import { MoveToFreeDialog } from "./move-to-free-dialog.tsx";
import { TierCard } from "./tier-card.tsx";
import "./billing.css";

export const plansStates = [
  "from-free",
  "on-team",
  "move-to-free",
  "move-to-free-unknown",
  "inactive-organization",
  "pending-checkout",
  "pending-confirming",
] as const;
export type PlansState = (typeof plansStates)[number];

const freeUse = usageScenes["free-over"];
const teamUse = usageScenes.steady;
const lastThirtyDays = savings.starts.own + savings.starts.boat;

type PlansContext = {
  current: TierKey;
  covers: TierKey;
  use: string;
  /** The Plan and usage screen that the breadcrumb opens. */
  usage: CataloguePath;
  /** A payment that waits, as the current API gives it. */
  pending?: "checkout" | "confirming";
};

const onTeam: PlansContext = {
  current: "team",
  covers: "team",
  use: `Your last 30 days: ${formatCount(lastThirtyDays)} starts on ${ownMachines(teamUse.machines)}.`,
  usage: paths.billing,
};

/** The use of 4 September in the words of the current API. */
const apiFreeUse = `This period so far: ${formatCount(freeUse.starts)} starts on ${ownMachines(freeUse.machines)}. At this pace, about ${formatCount(freeUse.projected)} starts by ${freePeriodEnd}.`;

const context: Record<PlansState, PlansContext> = {
  "from-free": {
    current: "free",
    covers: "team",
    use: `This month so far: ${formatCount(freeUse.starts)} starts on ${ownMachines(freeUse.machines)}. At this pace, about ${formatCount(freeUse.projected)} starts by ${freePeriodEnd}.`,
    usage: paths.usageFreeOver,
  },
  "on-team": onTeam,
  "move-to-free": onTeam,
  "move-to-free-unknown": onTeam,
  "inactive-organization": {
    current: "free",
    covers: "team",
    use: `Your workspace now: ${organizationsText(decemberOrganizationCount)} and ${ownMachines(inactiveUse.machines)}. ${tiers.free.name} includes ${organizationsText(tiers.free.organizations ?? 1)}.`,
    usage: inactivePages.usage,
  },
  "pending-checkout": {
    current: "free",
    covers: "team",
    use: apiFreeUse,
    usage: paths.usageCurrentContinue,
    pending: "checkout",
  },
  "pending-confirming": {
    current: "free",
    covers: "team",
    use: apiFreeUse,
    usage: paths.usageCurrentConfirming,
    pending: "confirming",
  },
};

const softLimits = {
  known:
    "Limits are soft: jobs keep running past them, and ciproxy offers the plan or add-on that covers your use. Prices in US dollars, billed monthly through Stripe.",
  api: "Limits are soft: jobs keep running past them, and ciproxy offers the plan or add-ons that cover your use. Prices in US dollars, billed monthly through Stripe.",
} as const;

/** The screen that a tier button opens, where the catalogue has one. */
function choicePath(
  state: PlansState,
  current: TierKey,
  tier: TierKey,
): CataloguePath | undefined {
  // The checkout of December has no screen, so its choices open none, and
  // no choice opens while a payment waits.
  if (state === "inactive-organization" || context[state].pending) {
    return undefined;
  }
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
  const { current, covers, use, usage, pending } = context[state];
  return (
    <AppShell
      viewport={viewport}
      tier={current}
      active="billing"
      role={role}
      banner={pending ? apiBanner({ scene: "free-over", usage }) : undefined}
    >
      <PageHeader
        crumbs={[{ label: "Plan and usage", to: usage }]}
        title="Plans"
        subtitle={use}
      />
      {pending ? <PendingNotice pending={pending} page="plans" /> : null}
      <div className="tier-grid">
        {tierOrder.map((key) => (
          <TierCard
            key={key}
            tier={tiers[key]}
            current={key === current}
            covers={key === covers && key !== current}
            to={choicePath(state, current, key)}
            canPay={canPay(role)}
            waiting={pending !== undefined}
            unitPrices={pending === undefined}
          />
        ))}
      </div>
      <p className="caption">{pending ? softLimits.api : softLimits.known}</p>
      {state === "move-to-free" ? <MoveToFreeDialog /> : null}
      {state === "move-to-free-unknown" ? <MoveToFreeDialog unknown /> : null}
    </AppShell>
  );
}
