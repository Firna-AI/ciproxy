import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../../screens/jobs/jobs-live.tsx";

export const loading = defineScreen({
  slug: "loading",
  title: "While jobs load",
  description:
    "Skeleton rows keep the layout of the list until the first read returns.",
  relatedDocs: [planDoc],
  address: orgAddress("jobs"),
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="loading" />
  ),
});

export const error = defineScreen({
  slug: "error",
  title: "When jobs fail to load",
  description:
    "The shared failed-read state with Try again. History uses the same state.",
  relatedDocs: [planDoc],
  address: orgAddress("jobs"),
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="error" />
  ),
});
