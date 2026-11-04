// Plan and usage: the tier, the starts and own machines of the period against
// the tier, the add-ons, and the way to the Stripe billing portal. A Viewer
// sees the same page without its payment actions, and can compare the plans.

import { ExternalLink } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { SkeletonBar, SkeletonGroup } from "@firna/ui/skeleton";

import { Action, LinkAction } from "../../components/actions.tsx";
import { ReadFailure } from "../../components/feedback.tsx";
import { Meter } from "../../components/figures.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AskPayer, canPay } from "../../components/role-actions.tsx";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { usageBanner } from "../../components/usage-banner.tsx";
import { dotJoin, formatCount, monthly } from "../../fixtures/format.ts";
import type { Role } from "../../fixtures/members.ts";
import {
  addOnPrices,
  ownMachines,
  startsText,
  tiers,
} from "../../fixtures/tiers.ts";
import {
  usageScenes,
  type Usage,
  type UsageScene,
} from "../../fixtures/usage.ts";
import { AddOns } from "./add-ons.tsx";
import { usageNotice } from "./usage-notices.tsx";
import "./billing.css";

export type PlanUsageState = UsageScene | "loading" | "error";

export function PlanUsageScreen({
  viewport,
  state,
  role = "admin",
}: {
  viewport: Viewport;
  state: PlanUsageState;
  /** The role of the signed-in person: a Viewer cannot pay. */
  role?: Role;
}) {
  const scene: UsageScene =
    state === "loading" || state === "error" ? "steady" : state;
  const usage = usageScenes[scene];
  const system = state === "loading" || state === "error";
  const payer = canPay(role);
  return (
    <AppShell
      viewport={viewport}
      tier={usage.tier}
      active="billing"
      role={role}
      banner={system ? undefined : usageBanner(scene, "billing", payer)}
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
          {usageNotice(scene, usage, payer)}
          <div className="grid-2">
            <CurrentPlan usage={usage} canPay={payer} />
            <UsagePanel usage={usage} />
          </div>
          <AddOns scene={scene} usage={usage} canPay={payer} />
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

function CurrentPlan({ usage, canPay }: { usage: Usage; canPay: boolean }) {
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
              to={paid ? paths.plansOnTeam : paths.plans}
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

/** The included starts and machines: the tier plus the saved add-ons. */
function limits(usage: Usage) {
  const tier = tiers[usage.tier];
  return {
    starts: tier.starts + usage.extraStarts * addOnPrices.startsBlock,
    machines: tier.machines + usage.extraMachines,
  };
}

function UsagePanel({ usage }: { usage: Usage }) {
  const limit = limits(usage);
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
