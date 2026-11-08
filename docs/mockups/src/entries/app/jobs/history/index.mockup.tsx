import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import {
  historyStates,
  JobsHistoryScreen,
} from "../../../../screens/jobs/jobs-history.tsx";

const shown = [
  "history",
  "empty",
  "no-match",
  "date-menu",
  "filtered",
] as const;

export const history = defineScreen({
  title: "History",
  description:
    "Finished jobs, newest first, with the result, where each ran, how long it waited for a runner, and its duration. A job name opens its detail. The filters stay above the list in every state.",
  relatedDocs: related("jobRouting", "billing"),
  address: orgAddress("jobs/history"),
  propSchema: stateSchema(historyStates),
  props: { state: "history" },
  render: (props, context) => (
    <JobsHistoryScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "empty",
      title: "No finished jobs yet",
      description:
        "A new organization before its first job finishes. The filters stay, so a person can choose a longer window.",
      props: { state: shown[1] },
    },
    {
      slug: "no-match",
      title: "No job matches the filters",
      description:
        "No Boat job of today matches deploy. The filters keep their values, and Clear filters opens the list of today.",
      props: { state: shown[2] },
    },
    {
      slug: "date-menu",
      title: "The date choices",
      description:
        "The choices are the history windows of the tiers that fit the 30 days of Team, in UTC days: Today, Last 7 days, and Last 30 days. Today is the default.",
      props: { state: shown[3] },
    },
    {
      slug: "filtered",
      title: "Jobs on Boat that match a search",
      description:
        "Boat and the search e2e leave the jobs that match. The footer counts the list and names the window of the plan.",
      props: { state: shown[4] },
    },
  ],
});
