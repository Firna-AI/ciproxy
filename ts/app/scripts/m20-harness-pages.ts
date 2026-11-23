// The pages of Jobs and Plans that the links of Machines and Settings open,
// as the fixture API of Machines and Settings answers them: History with no
// finished job, the page of each job that a busy slot names, whose figures
// show when it ends, and the plans of a workspace on Free. Every role reads
// them.

import type { components } from "../../api/index.ts";
import { historyDays, plansPage } from "./fixtures-billing.ts";
import { usageFixture, usageForRole } from "./fixtures-billing-usage.ts";
import { fixtureUsage } from "./fixtures-job-usage.ts";
import { problem } from "./harness-state.ts";
import type { FixtureReply, M20State } from "./m20-harness-state.ts";

type Schemas = components["schemas"];

/** The job that a busy slot of a machine of the scenario names. */
function slotJob(state: M20State, id: string): Schemas["JobView"] | undefined {
  for (const machine of state.machines)
    for (const slot of machine.occupancy)
      if (slot.job?.id === id) return slot.job;
  return undefined;
}

export function linkedPageReply(
  method: string,
  route: string,
  state: M20State,
  role: Schemas["WorkspaceRole"],
): FixtureReply | undefined {
  if (method !== "GET") return undefined;
  const usage = usageForRole(usageFixture("free"), role);
  if (route === "plans") return { status: 200, body: plansPage(usage) };
  if (route === "jobs/history") {
    const body: Schemas["JobsHistoryPage"] = {
      banner: null,
      history_days: historyDays(usage.status.selection.tier),
      jobs: [],
    };
    return { status: 200, body };
  }
  const match = /^jobs\/([^/]+)$/.exec(route);
  if (match === null) return undefined;
  const job = slotJob(state, decodeURIComponent(match[1] ?? ""));
  if (job === undefined)
    return { status: 404, body: problem("not_found", "Job not found.") };
  const body: Schemas["JobDetailPage"] = {
    job,
    banner: null,
    usage: fixtureUsage(job),
  };
  return { status: 200, body };
}
