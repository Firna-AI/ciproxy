// The key screens in one design direction: Machines, the live queue, and
// Savings, each with its mobile and desktop views. Each renders the product
// screen inside the scope of the direction, so the shared components draw
// the direction, and the main navigation stays inside it.

import { defineScreen } from "@mokly/mokly";

import { paths, type CataloguePath } from "../../components/paths.ts";
import { planDoc } from "../../components/state-schema.ts";
import { orgAddress } from "../../fixtures/org.ts";
import type {
  Direction,
  DirectionKey,
  DirectionRoutes,
} from "../../theme/direction.ts";
import { DirectionScope } from "../../theme/direction-scope.tsx";
import { JobsLiveScreen } from "../jobs/jobs-live.tsx";
import { MachinesScreen } from "../machines/machines-list.tsx";
import { SavingsScreen } from "../savings/savings.tsx";

type KeyScreens = {
  machines: CataloguePath;
  jobs: CataloguePath;
  savings: CataloguePath;
};

const keyScreens: Readonly<Record<DirectionKey, KeyScreens>> = {
  graphite: {
    machines: paths.graphiteMachines,
    jobs: paths.graphiteJobs,
    savings: paths.graphiteSavings,
  },
  midnight: {
    machines: paths.midnightMachines,
    jobs: paths.midnightJobs,
    savings: paths.midnightSavings,
  },
  enterprise: {
    machines: paths.enterpriseMachines,
    jobs: paths.enterpriseJobs,
    savings: paths.enterpriseSavings,
  },
};

/** The main navigation of a direction opens its own key screens. */
export function directionRoutes(key: DirectionKey): DirectionRoutes {
  const own = keyScreens[key];
  return {
    [paths.machines]: own.machines,
    [paths.jobs]: own.jobs,
    [paths.savings]: own.savings,
  };
}

/** Notes for reviewers: one line about the direction, and its reasoning. */
export type DirectionNotes = { line: string; rationale: string };

/** The three key screens of a direction, for one entry module to export. */
export function directionScreens(direction: Direction, notes: DirectionNotes) {
  const routes = directionRoutes(direction.key);
  const about = `${direction.name}: ${notes.line}`;
  const base = { relatedDocs: [planDoc], rationale: notes.rationale };
  return {
    machines: defineScreen({
      ...base,
      slug: "machines",
      title: "Machines",
      description: `${about} The three own machines on a busy afternoon.`,
      address: orgAddress("machines"),
      render: (_props, context) => (
        <DirectionScope direction={direction} routes={routes}>
          <MachinesScreen viewport={context.viewport} state="steady" />
        </DirectionScope>
      ),
    }),
    jobs: defineScreen({
      ...base,
      slug: "jobs",
      title: "Live jobs",
      description: `${about} The live queue: 21 jobs run, 15 on own machines and 6 on Boat.`,
      address: orgAddress("jobs"),
      render: (_props, context) => (
        <DirectionScope direction={direction} routes={routes}>
          <JobsLiveScreen viewport={context.viewport} state="steady" />
        </DirectionScope>
      ),
    }),
    savings: defineScreen({
      ...base,
      slug: "savings",
      title: "Savings",
      description: `${about} The last 30 days against Blacksmith and GitHub-hosted runners.`,
      address: orgAddress("savings"),
      render: (_props, context) => (
        <DirectionScope direction={direction} routes={routes}>
          <SavingsScreen viewport={context.viewport} state="report" />
        </DirectionScope>
      ),
    }),
  };
}
