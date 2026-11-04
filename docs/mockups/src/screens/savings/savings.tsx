// What CI cost with ciproxy in the last 30 days, against the same jobs on
// Blacksmith and on GitHub-hosted runners. Every role sees the report; only
// an Admin enters what a machine costs, so a Viewer or a Billing person reads
// Ask an Admin in place of Add costs.

import { PiggyBank } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { SkeletonBar, SkeletonGroup } from "@firna/ui/skeleton";
import { SegmentedControl } from "@firna/ui/segmented";

import { LinkAction, noop } from "../../components/actions.tsx";
import { Notice, ReadFailure, StateBlock } from "../../components/feedback.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AskAdmin } from "../../components/role-actions.tsx";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { StartsChart } from "../../components/starts-chart.tsx";
import { formatCount, formatMoney, unbroken } from "../../fixtures/format.ts";
import type { Role } from "../../fixtures/members.ts";
import { listPrices } from "../../fixtures/price-lists.ts";
import {
  dailyStarts,
  savings,
  savingsPeriod,
  yourCost,
} from "../../fixtures/savings.ts";
import { CostBars } from "./cost-bars.tsx";
import { CostTable } from "./cost-table.tsx";
import "./savings.css";

export const savingsStates = [
  "report",
  "missing-costs",
  "empty",
  "loading",
  "error",
] as const;
export type SavingsState = (typeof savingsStates)[number];

/** The days of the price lists that the report uses, once for each day. */
const priceListDays = [
  ...new Set([listPrices.blacksmith.checked, listPrices.github.checked]),
]
  .map(unbroken)
  .join(" and ");

const periods = [
  { label: "Last 30 days", value: "30" },
  { label: "This month", value: "month" },
  { label: "Last month", value: "last" },
] as const;

export function SavingsScreen({
  viewport,
  state,
  role = "admin",
}: {
  viewport: Viewport;
  state: SavingsState;
  /** The role of the signed-in person. */
  role?: Role;
}) {
  return (
    <AppShell
      viewport={viewport}
      tier={state === "empty" ? "free" : "team"}
      active="savings"
      role={role}
      setup={
        state === "empty" ? { done: 3, total: 4, active: false } : undefined
      }
    >
      <PageHeader
        title="Savings"
        subtitle={
          state === "empty" ? undefined : `Last 30 days · ${savingsPeriod}`
        }
      />
      {state === "empty" || state === "error" ? null : (
        <SegmentedControl
          options={periods}
          value="30"
          onChange={noop}
          accessibilityLabel="Period"
        />
      )}
      <SavingsBody state={state} role={role} />
    </AppShell>
  );
}

function SavingsBody({ state, role }: { state: SavingsState; role: Role }) {
  if (state === "error") {
    return (
      <Panel>
        <ReadFailure subject="the savings report" retry={paths.savings} />
      </Panel>
    );
  }
  if (state === "empty") {
    return (
      <Panel>
        <StateBlock
          icon={PiggyBank}
          title="Your savings report starts with your first job"
          action={
            <LinkAction to={paths.onboarding} tone="primary">
              Set up ciproxy
            </LinkAction>
          }
        >
          It compares what your jobs cost with the same jobs on GitHub-hosted
          runners and on Blacksmith.
        </StateBlock>
      </Panel>
    );
  }
  if (state === "loading") return <LoadingSavings />;
  const missing = state === "missing-costs";
  const saved = savings.alternatives.blacksmith - yourCost;
  const jobs = savings.starts.own + savings.starts.boat;
  return (
    <>
      {missing ? (
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
      <Panel>
        {missing ? null : (
          <div className="hero">
            <p className="hero-figure">{formatMoney(saved)}</p>
            <p className="hero-text">
              less than Blacksmith for the same {formatCount(jobs)} jobs, and{" "}
              {formatMoney(savings.alternatives.github - yourCost)} less than{" "}
              <span className="nowrap">GitHub-hosted</span> runners.
            </p>
          </div>
        )}
        <CostBars withCiproxy={!missing} />
        <p className="caption cost-note">
          Blacksmith and GitHub-hosted costs use their list prices per minute
          for the same VM sizes, from the price lists of {priceListDays}.
        </p>
      </Panel>
      <div className="grid-2">
        <Panel title="Your cost with ciproxy" flush>
          <CostTable missing={missing} />
        </Panel>
        <Panel title="Starts by day">
          <StartsChart
            labels={dailyStarts.labels}
            own={dailyStarts.own}
            boat={dailyStarts.boat}
            caption={savingsPeriod}
          />
        </Panel>
      </div>
    </>
  );
}

function LoadingSavings() {
  return (
    <Panel>
      <SkeletonGroup direction="column" gap={14}>
        <SkeletonBar width={140} height={44} />
        <SkeletonBar width="70%" height={14} />
        <SkeletonBar width="90%" height={20} />
        <SkeletonBar width="60%" height={20} />
        <SkeletonBar width="40%" height={20} />
      </SkeletonGroup>
    </Panel>
  );
}
