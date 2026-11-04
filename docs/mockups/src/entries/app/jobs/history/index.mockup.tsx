import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobsHistoryScreen } from "../../../../screens/jobs/jobs-history.tsx";

export const history = defineScreen({
  title: "History",
  description:
    "Finished jobs, newest first, with the result, where each ran, how long it waited for a runner, and its duration. A job name opens its detail.",
  relatedDocs: related("jobRouting", "billing"),
  address: orgAddress("jobs/history"),
  propSchema: stateSchema(["history", "empty"]),
  props: { state: "history" },
  render: (props, context) => (
    <JobsHistoryScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "empty",
      title: "No finished jobs yet",
      description: "A new organization before its first job finishes.",
      props: { state: "empty" },
    },
  ],
});
