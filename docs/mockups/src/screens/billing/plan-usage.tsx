// Plan and usage: the tier, the starts and own machines of the period against
// the tier, the add-ons, and the way to the Stripe billing portal. A Viewer
// sees the same page without its payment actions, and can compare the plans.
// The banner and the links of a moment come from billing-scenes.tsx. With
// `api`, the page shows what the current API gives at that moment: its
// banner, its notices, and the add-ons that it prices.

import { ExternalLink } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { SkeletonBar, SkeletonGroup } from "@firna/ui/skeleton";

import { Action, LinkAction } from "../../components/actions.tsx";
import { apiBanner } from "../../components/api-banner.tsx";
import { ReadFailure } from "../../components/feedback.tsx";
import { Meter } from "../../components/figures.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { AskPayer, canPay } from "../../components/role-actions.tsx";
import { AppShell } from "../../components/shell/app-shell.tsx";
import type { AddOnCount } from "../../fixtures/current-api.ts";
import { dotJoin, formatCount, monthly } from "../../fixtures/format.ts";
import type { Role } from "../../fixtures/members.ts";
import { ownMachines, startsText, tiers } from "../../fixtures/tiers.ts";
import {
  included,
  usageScenes,
  type Usage,
  type UsageScene,
} from "../../fixtures/usage.ts";
import { AddOns } from "./add-ons.tsx";
import { ApiAddOns, type AddOnsOutcome } from "./api-add-ons.tsx";
import {
  PaceNotice,
  PendingNotice,
  type PendingPayment,
} from "./api-notices.tsx";
import { freePlansOf, sceneBanner } from "./billing-scenes.tsx";
import { usageNotice } from "./usage-notices.tsx";
import "./billing.css";

export type PlanUsageState = UsageScene | "loading" | "error";

/** Plan and usage as the current API gives it, at one moment. */
export type UsageApi = {
  /** The Plans screen of the moment, which the banner and the plan open. */
  plans: CataloguePath;
  /** A payment that waits, and the Stripe page that this tab saved. */
  pending?: PendingPayment;
  /** The add-on quantities that the payer set, when not the saved ones. */
  edited?: AddOnCount;
  /** What Stripe answered to Save add-ons. */
  outcome?: AddOnsOutcome;
};

export function PlanUsageScreen({
  viewport,
  state,
  role = "admin",
  api,
}: {
  viewport: Viewport;
  state: PlanUsageState;
  /** The role of the signed-in person: a Viewer cannot pay. */
  role?: Role;
  api?: UsageApi;
}) {
  const scene: UsageScene =
    state === "loading" || state === "error" ? "steady" : state;
  const usage = usageScenes[scene];
  const system = state === "loading" || state === "error";
  const payer = canPay(role);
  const plans = api?.plans ?? freePlansOf(scene);
  const banner = api
    ? apiBanner({ scene, plans: api.plans })
    : sceneBanner(scene, "billing", payer);
  return (
    <AppShell
      viewport={viewport}
      tier={usage.tier}
      active="billing"
      role={role}
      banner={system ? undefined : banner}
    >
      <PageHeader title="Plan and usage" />
      {state === "error" ? (
        <Panel>
          <ReadFailure subject="plan and usage" retry={paths.billing} />
        </Panel>
      ) : state === "loading" ? (
        <LoadingUsage />
      ) : (
        <>
          {api ? (
            <>
              {api.pending ? (
                <PendingNotice pending={api.pending} page="usage" />
              ) : null}
              <PaceNotice scene={scene} plans={api.plans} />
            </>
          ) : (
            usageNotice(scene, usage, payer, plans)
          )}
          <div className="grid-2">
            <CurrentPlan usage={usage} canPay={payer} plans={plans} />
            <UsagePanel usage={usage} />
          </div>
          {api ? (
            <ApiAddOns
              scene={scene}
              edited={api.edited}
              outcome={api.outcome}
              waiting={api.pending !== undefined || api.outcome !== undefined}
            />
          ) : (
            <AddOns scene={scene} usage={usage} canPay={payer} />
          )}
        </>
      )}
    </AppShell>
  );
}

/** When a paid tier renews, or the day it ends after a move to Free. */
function renewal(usage: Usage): string | undefined {
  if (usage.ends) return `ends on ${usage.ends}`;
  return usage.renews ? `renews on ${usage.renews}` : undefined;
}

function CurrentPlan({
  usage,
  canPay,
  plans,
}: {
  usage: Usage;
  canPay: boolean;
  /** The Plans screen that Compare plans opens on Free. */
  plans: CataloguePath;
}) {
  const tier = tiers[usage.tier];
  const paid = tier.price > 0;
  const when = paid ? renewal(usage) : undefined;
  return (
    <Panel title="Plan">
      <div className="stack">
        <div className="stack-tight">
          <p className="plan-name">{tier.name}</p>
          <p className="muted">
            {when ? dotJoin(monthly(tier.price), when) : monthly(tier.price)}
          </p>
          <p className="small">
            {startsText(tier.starts)} a month and {ownMachines(tier.machines)}
          </p>
        </div>
        {canPay ? (
          <div className="row-wrap">
            <LinkAction
              to={paid ? paths.plansOnTeam : plans}
              tone={paid ? "secondary" : "primary"}
            >
              {paid ? "Change plan" : "Compare plans"}
            </LinkAction>
            {paid ? (
              <Action icon={ExternalLink}>Invoices and card</Action>
            ) : null}
          </div>
        ) : (
          <div className="row-wrap">
            <LinkAction to={paths.plansViewer}>Compare plans</LinkAction>
            <AskPayer />
          </div>
        )}
        {paid && canPay ? (
          <p className="caption">
            Invoices and the card are in the Stripe billing portal.
          </p>
        ) : null}
      </div>
    </Panel>
  );
}

function UsagePanel({ usage }: { usage: Usage }) {
  const limit = included(usage);
  const startsOver = usage.starts > limit.starts;
  const startsNear = usage.projected > limit.starts;
  const machinesOver = usage.machines > limit.machines;
  return (
    <Panel title="This period">
      <div className="stack">
        <p className="caption">{usage.period}</p>
        <Meter
          label="Starts"
          used={usage.starts}
          max={limit.starts}
          tone={startsOver || startsNear ? "warning" : "normal"}
          value={`${formatCount(usage.starts)} of ${formatCount(limit.starts)}`}
          detail={
            startsOver
              ? `${formatCount(usage.starts - limit.starts)} over. Jobs keep running.`
              : `About ${formatCount(usage.projected)} by the end of the period at this pace.`
          }
        />
        <Meter
          label="Own machines"
          used={usage.machines}
          max={limit.machines}
          tone={machinesOver ? "warning" : "normal"}
          value={`${usage.machines} of ${limit.machines}`}
          detail={
            machinesOver
              ? `${usage.machines - limit.machines} over. Every machine runs jobs.`
              : undefined
          }
        />
        <p className="caption">
          Counts the starts and machines of every organization of this
          workspace.
        </p>
      </div>
    </Panel>
  );
}

function LoadingUsage() {
  return (
    <div className="grid-2">
      {[0, 1].map((key) => (
        <Panel key={key}>
          <SkeletonGroup direction="column" gap={12}>
            <SkeletonBar width="40%" height={16} />
            <SkeletonBar width="70%" height={12} />
            <SkeletonBar width="100%" height={8} />
            <SkeletonBar width="100%" height={8} />
          </SkeletonGroup>
        </Panel>
      ))}
    </div>
  );
}
