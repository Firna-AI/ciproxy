import { defineScreen } from "@mokly/mokly";

import { related, relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../../screens/jobs/jobs-live.tsx";

const address = orgAddress("jobs");

export const boatRefused = defineScreen({
  slug: "boat-refused",
  title: "Boat refuses new sandboxes",
  description:
    "A peak on 25 November fills the start limit of the Boat plan. Boat refuses new sandboxes until 10:13, so 9 jobs wait past the 30 seconds. Each one takes the first free slot on an own machine, and ciproxy tries Boat again.",
  rationale:
    "A refused start is a limit of the Boat account, not of ciproxy, so the recovery is on Boat. The jobs that run keep running.",
  relatedDocs: related("overflow", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="boat-refused" />
  ),
});

export const nothingFits = defineScreen({
  slug: "nothing-fits",
  title: "Jobs of a label that nothing fits",
  description:
    "Two load tests use ciproxy-32vcpu, which no own machine and no Boat size fits. The label is set to wait when nothing fits, so the jobs stay queued in GitHub. Edit label opens its dialog.",
  relatedDocs: related("labels", "overflow", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="nothing-fits" />
  ),
});

export const appSuspended = defineScreen({
  slug: "app-suspended",
  title: "The App is suspended",
  description:
    "Tom Becker, an owner of mokly-ai, suspended the ciproxy GitHub App on that organization on 19 November. No new job of mokly-ai starts on a ciproxy runner, and the 7 jobs that started before finish. ciproxy stopped the warm runners and cannot see the jobs of mokly-ai that wait on GitHub, so the figures leave out Waiting. Jobs of mokly-labs, the other organization of the workspace, start as before. Machines, Boat, the plan, and the data stay as they are. Open GitHub leads to the install, where an owner unsuspends it.",
  rationale:
    "A suspension changes nothing but the jobs, because the owner can undo it at any time. It shows here, with the other reasons why jobs wait. The removed App has its own page under The App on GitHub.",
  relatedDocs: relatedMvp("appRemoval", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="app-suspended" />
  ),
});
