// The job pages of facts that the current API gives or lacks, on Wednesday
// 18 November: a job that no ciproxy runner took, a job whose finish time,
// VM, and cache GitHub and ciproxy do not know, and two jobs as the current
// API shows them. The API gives no routing reason yet, so these pages say
// that the reason is not available. Their timelines hold only the times that
// the API gives: queued, taken, and finished. The subtitle names no pull
// request title, because the API gives none.

import type { JobDetail } from "./job-details.ts";
import { none, onBoat, own, repositoryName } from "./jobs.ts";
import { labelNamed } from "./settings.ts";
import { boatSizeFor } from "./sizes.ts";

const label = labelNamed("ciproxy");

export const factDetails = {
  "no-runner": {
    name: "e2e (webkit) 2/2",
    context: `${repositoryName("mokly")} · E2E · #1537`,
    ref: "#1537",
    where: none,
    label: label.name,
    request: "unknown",
    why: "No ciproxy runner took this job, so it is not a start.",
    result: "cancelled",
    noRunner: true,
    timeline: [
      { time: "14:20:12", text: "Queued on GitHub" },
      { time: "14:21:03", text: "Cancelled" },
    ],
    waited: "Unknown",
    duration: "Unknown",
    cache: { source: "unknown" },
  },
  "unknown-facts": {
    name: "deploy preview",
    context: `${repositoryName("mokly-cloud")} · Preview · #320`,
    ref: "#320",
    where: own(3, 1),
    label: label.name,
    request: "unknown",
    why: null,
    timeline: [
      { time: "14:22:40", text: "Queued on GitHub" },
      {
        time: "14:22:40",
        text: "Taken by a runner on",
        machine: "hetzner-ax42-3",
      },
    ],
    waited: "0s",
    duration: "Unknown",
    cache: { source: "unknown" },
  },
  "current-own": {
    name: "test (rust)",
    context: `${repositoryName("mokly")} · CI · #1542`,
    ref: "#1542",
    where: own(1, 3),
    label: label.name,
    request: label,
    why: null,
    currentContract: true,
    timeline: [
      { time: "14:25:09", text: "Queued on GitHub" },
      {
        time: "14:25:09",
        text: "Taken by a runner on",
        machine: "hetzner-ax42-1",
      },
      { time: "14:31:21", text: "Passed" },
    ],
    waited: "0s",
    duration: "6m 12s",
    cache: { source: "machine", restored: 3, sizeGb: 2.1 },
  },
  "current-boat": {
    name: "e2e (chromium) 3/4",
    context: `${repositoryName("mokly")} · E2E · main`,
    ref: "main",
    where: onBoat,
    label: label.name,
    request: label,
    boatSize: boatSizeFor(label),
    why: null,
    currentContract: true,
    timeline: [
      { time: "14:23:04", text: "Queued on GitHub" },
      { time: "14:23:35", text: "Taken by a runner on Boat" },
      { time: "14:30:40", text: "Passed" },
    ],
    waited: "31s",
    duration: "7m 05s",
    cache: { source: "github", fallback: false },
  },
} as const satisfies Readonly<Record<string, JobDetail>>;
