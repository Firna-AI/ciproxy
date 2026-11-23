import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related, relatedMvp } from "../../../../components/related-docs.ts";
import { stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../screens/onboarding/onboarding.tsx";

export const firstJob = defineScreen({
  title: "Run the first job",
  description:
    "The last step. The page shows the one-line change to runs-on and waits for the first job with the ciproxy label.",
  relatedDocs: related("labels", "jobRouting", "overflowSpend"),
  address: orgAddress("setup"),
  useCasePaths: [paths.flowFirstGreenJob],
  propSchema: stateSchema([
    "first-job-waiting",
    "first-job-passed",
    "provider-skipped",
    "first-job-preparing",
  ]),
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
    {
      slug: "after-skip",
      title: "After Skip for now",
      description:
        "The provider step shows a dash, does not count as done, and links to Settings under Overflow. The first job is open. Setup completes with the first job, also without a provider.",
      rationale:
        "The API keeps no record of a skip, so a skip lasts only for the browser tab. Setup ends with the first job, so the provider step never holds a finished Setup open.",
      relatedDocs: related("labels", "jobRouting"),
      props: { state: "provider-skipped" },
      useCasePaths: [paths.flowSkipBoat],
    },
    {
      slug: "waits-for-image",
      title: "The first job waits for the job image",
      description:
        "Cal Moore skipped the provider step and pushed the change while hetzner-ax42-1 still gets its first job image. test (rust) waits for a runner, and the notice says why: the job starts on the machine when it is ready. The closed machine step names the image too.",
      rationale:
        "The answer E to item 124: the first-job step gives the reason while a first job waits for a new machine. A machine without its image takes no job, so with a provider connected the job would start on the next place of its label at once; the job waits for the machine only without a provider. No text promises a time, because a slow link makes the download long.",
      relatedDocs: relatedMvp("agent", "jobRouting", "overflow"),
      props: { state: "first-job-preparing" },
    },
  ],
});
