// The job page of a what-if with two providers, on Wednesday 18 November at
// 14:19: the order of ciproxy is the own machines, Boat, then Second
// provider, and the start limit of the Boat plan is full. A job waits the
// overflow wait for a free own slot, skips Boat, which refuses new
// sandboxes, and starts on Second provider at once, with no extra wait
// between the providers, as the answer to item 4 of the provider plan says.
// The history of the same what-if lists the job.

import type { JobDetail } from "./job-details.ts";
import { onSecond, repositoryName } from "./jobs.ts";
import { overflow } from "./overflow.ts";
import { labelNamed } from "./settings.ts";
import { providerSizeFor } from "./sizes.ts";

const label = labelNamed("ciproxy");

export const providerDetails = {
  "second-provider": {
    name: "docker image",
    context: `${repositoryName("mokly-cloud")} · Release · main`,
    ref: "main",
    where: onSecond,
    label: label.name,
    request: label,
    providerSize: providerSizeFor("second", label),
    why: `Your machines were full for ${overflow.waitSeconds} seconds, and Boat refused new sandboxes. The job started on Second provider, the next place in the order of its label.`,
    timeline: [
      { time: "14:19:08", text: "Queued on GitHub" },
      { time: "14:19:08", text: "No free slot on your machines" },
      { time: "14:19:38", text: "Boat refused new sandboxes" },
      { time: "14:19:38", text: "Runner started on Second provider" },
      { time: "14:19:40", text: "Taken by the Second provider runner" },
      { time: "14:24:20", text: "Passed" },
    ],
    waited: "32s",
    duration: "4m 40s",
    cache: { source: "github", fallback: false },
  },
} as const satisfies Readonly<Record<string, JobDetail>>;
