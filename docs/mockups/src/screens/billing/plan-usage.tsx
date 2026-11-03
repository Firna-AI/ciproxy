// Plan and usage: the tier, the starts and own nodes of the period against the
// tier, the add-ons, and the way to the Stripe billing portal.

import { ExternalLink } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { SkeletonBar, SkeletonGroup } from "@firna/ui/skeleton";

import { Action, LinkAction } from "../../components/actions.tsx";
import { ReadFailure } from "../../components/feedback.tsx";
import { Meter } from "../../components/figures.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { formatCount, formatMoney } from "../../fixtures/format.ts";
import { addOnPrices, tiers } from "../../fixtures/tiers.ts";
import {
  usageScenes,
  type Usage,
  type UsageScene,
} from "../../fixtures/usage.ts";
import { AddOns } from "./add-ons.tsx";
import { usageBanner, usageNotice } from "./usage-notices.tsx";
import "./billing.css";

export type PlanUsageState = UsageScene | "loading" | "error";

export function PlanUsageScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: PlanUsageState;
}) {
  const scene: UsageScene =
    state === "loading" || state === "error" ? "steady" : state;
  const usage = usageScenes[scene];
  const system = state === "loading" || state === "error";
  return (
    <AppShell
      viewport={viewport}
      tier={usage.tier}
      active="billing"
      banner={system ? undefined : usageBanner(scene, usage)}
    >
      <PageHeader title="Plan and usage" />
      {state === "error" ? (
        <Panel>
          <ReadFailure retry={paths.billing} />
        </Panel>
      ) : state === "loading" ? (
        <LoadingUsage />
      ) : (
        <>
          {usageNotice(scene, usage)}
          <div className="grid-2">
            <CurrentPlan usage={usage} />
            <UsagePanel usage={usage} />
          </div>
          <AddOns scene={scene} usage={usage} />
        </>
      )}
    </AppShell>
  );
}

function CurrentPlan({ usage }: { usage: Usage }) {
  const tier = tiers[usage.tier];
  const paid = tier.price > 0;
  return (
    <Panel title="Plan">
      <div className="stack">
        <div className="stack-tight">
          <p className="plan-name">{tier.name}</p>
          <p className="muted">
            {paid
              ? `${formatMoney(tier.price)} a month · renews on ${usage.renews}`
              : "$0"}
          </p>
          <p className="small">
            {formatCount(tier.starts)} starts and {tier.nodes} own{" "}
            {tier.nodes === 1 ? "node" : "nodes"} a month
          </p>
        </div>
        <div className="row-wrap">
          <LinkAction
            to={paid ? paths.plansOnTeam : paths.plans}
            tone={paid ? "secondary" : "primary"}
          >
            {paid ? "Change plan" : "Compare plans"}
          </LinkAction>
          {paid ? <Action icon={ExternalLink}>Invoices and card</Action> : null}
        </div>
        {paid ? (
          <p className="caption">
            Invoices and the card are in the Stripe billing portal.
          </p>
        ) : null}
      </div>
    </Panel>
  );
}

/** The included starts and nodes: the tier plus the saved add-ons. */
function limits(usage: Usage) {
  const tier = tiers[usage.tier];
  return {
    starts: tier.starts + usage.extraStarts * addOnPrices.startsBlock,
    nodes: tier.nodes + usage.extraNodes,
  };
}

function UsagePanel({ usage }: { usage: Usage }) {
  const limit = limits(usage);
  const startsOver = usage.starts > limit.starts;
  const startsNear = usage.projected > limit.starts;
  const nodesOver = usage.nodes > limit.nodes;
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
          label="Own nodes"
          used={usage.nodes}
          max={limit.nodes}
          tone={nodesOver ? "warning" : "normal"}
          value={`${usage.nodes} of ${limit.nodes}`}
          detail={
            nodesOver
              ? `${usage.nodes - limit.nodes} over. Every node runs jobs.`
              : undefined
          }
        />
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
