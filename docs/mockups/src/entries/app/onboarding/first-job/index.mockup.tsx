import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../screens/onboarding/onboarding.tsx";

export const firstJob = defineScreen({
  title: "Run the first job",
  description:
    "The last step. The page shows the one-line change to runs-on and waits for the first job with the ciproxy label.",
  relatedDocs: related("labels", "jobRouting"),
  address: orgAddress("setup"),
  useCasePaths: [paths.flowFirstGreenJob],
  propSchema: stateSchema(["first-job-waiting", "first-job-passed"]),
  props: { state: "first-job-waiting" },
  render: (props, context) => (
    <OnboardingScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "passed",
      title: "The first job passed",
      description:
        "The first job ran on hetzner-ax42-1 and passed. Setup is complete, and Open Jobs leads to the live queue.",
      props: { state: "first-job-passed" },
      useCasePaths: [paths.flowFirstGreenJob],
    },
  ],
});
