// What CI cost with ciproxy in a UTC range, against the same jobs on
// Blacksmith and on GitHub-hosted runners. Every role sees the report; only
// an Admin enters what a machine costs, so a Viewer or a Billing person reads
// Ask an Admin in place of Add costs. The range control stays above the
// report, also when a range has no jobs. The savings figure shows only when
// every cost is known, and it says "more" when ciproxy costs more.

import { PiggyBank } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { SkeletonBar, SkeletonGroup } from "@firna/ui/skeleton";
import { SegmentedControl } from "@firna/ui/segmented";

import { LinkAction, noop } from "../../components/actions.tsx";
import { ReadFailure, StateBlock } from "../../components/feedback.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { StartsChart } from "../../components/starts-chart.tsx";
import { formatCount, formatMoney, unbroken } from "../../fixtures/format.ts";
import type { Role } from "../../fixtures/members.ts";
import { listPrices } from "../../fixtures/price-lists.ts";
import {
  leadOf,
  savingsFigures,
  totalCost,
  type Alternative,
  type ReportFigures,
  type ReportRange,
  type SavingsState,
} from "../../fixtures/savings-states.ts";
import { CostBars } from "./cost-bars.tsx";
import { CostTable } from "./cost-table.tsx";
import { SavingsNotices } from "./savings-notices.tsx";
import "./savings.css";

export type { SavingsState };

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

const rangeLabels: Readonly<Record<ReportRange, string>> = {
  "30": "Last 30 days",
  month: "This month",
  last: "Last month",
};

const names: Readonly<Record<Alternative, string>> = {
  blacksmith: "Blacksmith",
  github: "GitHub-hosted runners",
};

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
  const figures = savingsFigures[state];
  const read = state !== "loading" && state !== "error";
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
          read ? `${rangeLabels[figures.range]} · ${figures.period}` : undefined
        }
      />
      {state === "error" ? null : (
        <SegmentedControl
          options={periods}
          value={figures.range}
          onChange={noop}
          accessibilityLabel="Period"
        />
      )}
      <SavingsBody state={state} figures={figures} role={role} />
    </AppShell>
  );
}

/** "$390.50 less than Blacksmith for the same 17,760 jobs, and …". */
function Hero({ figures }: { figures: ReportFigures }) {
  const lead = leadOf(figures);
  const total = totalCost(figures);
  if (lead === null || total === null) return null;
  const other: Alternative = lead === "blacksmith" ? "github" : "blacksmith";
  const saved = (alternative: Alternative) =>
    (figures.alternatives[alternative] ?? 0) - total;
  const words = (value: number) => (value < 0 ? "more than" : "less than");
  const jobs = figures.starts.own + figures.starts.boat;
  return (
    <div className="hero">
      <p className="hero-figure">{formatMoney(Math.abs(saved(lead)))}</p>
      <p className="hero-text">
        {words(saved(lead))} {names[lead]} for the same {formatCount(jobs)}{" "}
        jobs, and {formatMoney(Math.abs(saved(other)))} {words(saved(other))}{" "}
        <span className="nowrap">{names[other]}</span>.
      </p>
    </div>
  );
}

function EmptyReport({
  figures,
  role,
}: {
  figures: ReportFigures;
  role: Role;
}) {
  if (figures.range !== "30") {
    return (
      <StateBlock
        icon={PiggyBank}
        title={`No jobs ran ${rangeLabels[figures.range].toLowerCase()}`}
      >
        Choose another range to see your savings.
      </StateBlock>
    );
  }
  return (
    <StateBlock
      icon={PiggyBank}
      title="Your savings report starts with your first job"
      action={
        role === "admin" ? (
          <LinkAction to={paths.onboarding} tone="primary">
            Set up ciproxy
          </LinkAction>
        ) : undefined
      }
    >
      It compares what your jobs cost with the same jobs on GitHub-hosted
      runners and on Blacksmith.
    </StateBlock>
  );
}

function SavingsBody({
  state,
  figures,
  role,
}: {
  state: SavingsState;
  figures: ReportFigures;
  role: Role;
}) {
  if (state === "error") {
    return (
      <Panel>
        <ReadFailure subject="the savings report" retry={paths.savings} />
      </Panel>
    );
  }
  if (state === "loading") return <LoadingSavings />;
  if (figures.starts.own + figures.starts.boat === 0) {
    return (
      <Panel>
        <EmptyReport figures={figures} role={role} />
      </Panel>
    );
  }
  return (
    <>
      <SavingsNotices figures={figures} role={role} />
      <Panel>
        <Hero figures={figures} />
        <CostBars figures={figures} />
        <p className="caption cost-note">
          Blacksmith and GitHub-hosted costs use their list prices per minute
          for the same VM sizes, from the price lists of {priceListDays}.
        </p>
      </Panel>
      <div className="grid-2">
        <Panel title="Your cost with ciproxy" flush>
          <CostTable figures={figures} />
        </Panel>
        <Panel title="Starts by day">
          <StartsChart
            labels={figures.daily.labels}
            own={figures.daily.own}
            boat={figures.daily.boat}
            caption={figures.period}
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
