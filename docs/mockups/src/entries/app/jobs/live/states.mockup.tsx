import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { planDoc } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../../screens/jobs/jobs-live.tsx";

const address = orgAddress("jobs");

export const peak = defineScreen({
  slug: "peak",
  title: "A peak overflows to Boat",
  description:
    "A release at 10:05: every own slot is busy and 12 jobs wait. ciproxy starts one Boat runner for each job that has waited 30 seconds.",
  rationale:
    "A waiting job shows no machine: GitHub gives it to whichever runner is idle first, so ciproxy reports the runners it starts, not a place for each job.",
  relatedDocs: [planDoc],
  address,
  useCasePaths: [paths.flowPeakOverflow],
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="peak" />
  ),
});

export const spendLimit = defineScreen({
  slug: "spend-limit",
  title: "Boat reached the spend limit",
  description:
    "Boat spend reached the monthly limit that the customer set. Jobs run on own machines only and wait longer for a slot; Raise the limit opens the overflow settings.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="spend-limit" />
  ),
});

export const startsPassed = defineScreen({
  slug: "starts-passed",
  title: "Free starts passed",
  description:
    "On 4 September mokly-ai, still on Free, passes its 2,000 starts. Jobs keep running; the banner on every page offers Team, which covers the use.",
  relatedDocs: [planDoc],
  address,
  useCasePaths: [paths.flowFreeToTeam],
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="starts-passed" />
  ),
});

export const quiet = defineScreen({
  slug: "quiet",
  title: "Nothing runs",
  description:
    "Sunday night: no job runs or waits. The page says that warm runners are ready, so the next job starts at once.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="quiet" />
  ),
});
