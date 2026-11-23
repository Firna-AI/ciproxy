// The jobs of the CPU and memory states of the job page, on Wednesday 18
// November. visual diff failed on hetzner-ax42-3, because a step ran out of
// memory. In the what-ifs of macOS jobs, test (safari) ran on mac-studio-1.
// test (rust) shows while it runs, before its figures come. typecheck ran on
// hetzner-ax42-1 with an agent from before the figures, so it has none. The
// steps of unit (node 24) could not be read, so it has its figures without
// steps. History opens the pages of visual diff, typecheck, and unit (node
// 24) from their rows.

import type { JobDetail } from "./job-details.ts";
import { own, repositoryName, warmReason } from "./jobs.ts";
import { macStudioMachine } from "./macs.ts";
import { labelNamed, macosLabel } from "./settings.ts";

const label = labelNamed("ciproxy");

export const usageDetails = {
  "out-of-memory": {
    name: "visual diff",
    context: `${repositoryName("mokly")} · CI · #1538 Restyle the catalogue sidebar`,
    ref: "#1538",
    where: own(3, 2),
    label: label.name,
    request: label,
    why: warmReason,
    result: "failed",
    timeline: [
      { time: "14:23:56", text: "Queued on GitHub" },
      {
        time: "14:23:57",
        text: "Taken by a warm runner on",
        machine: "hetzner-ax42-3",
      },
      { time: "14:27:09", text: "Failed" },
    ],
    waited: "1s",
    duration: "3m 12s",
    cache: { source: "machine", restored: 1, sizeGb: 0.4 },
  },
  macos: {
    name: "test (safari)",
    context: `${repositoryName("mokly")} · CI · #1542 Add catalogue search`,
    ref: "#1542",
    where: { kind: "own", machine: macStudioMachine.name, slot: 1 },
    label: macosLabel.name,
    request: macosLabel,
    why: warmReason,
    timeline: [
      { time: "14:21:40", text: "Queued on GitHub" },
      {
        time: "14:21:40",
        text: "Taken by a warm runner on",
        machine: macStudioMachine.name,
      },
      { time: "14:29:28", text: "Passed" },
    ],
    waited: "0s",
    duration: "7m 48s",
    cache: { source: "machine", restored: 2, sizeGb: 1.3 },
  },
  running: {
    name: "test (rust)",
    context: `${repositoryName("mokly")} · CI · #1542 Add catalogue search`,
    ref: "#1542",
    where: own(1, 3),
    label: label.name,
    request: label,
    why: warmReason,
    result: "running",
    timeline: [
      { time: "14:25:09", text: "Queued on GitHub" },
      {
        time: "14:25:09",
        text: "Taken by a warm runner on",
        machine: "hetzner-ax42-1",
      },
    ],
    waited: "0s",
    duration: "4m 12s",
    cache: { source: "machine", restored: 3, sizeGb: 2.1 },
  },
  "no-figures": {
    name: "typecheck",
    context: `${repositoryName("mokly")} · CI · #1538 Restyle the catalogue sidebar`,
    ref: "#1538",
    where: own(1, 6),
    label: label.name,
    request: label,
    why: warmReason,
    timeline: [
      { time: "14:21:22", text: "Queued on GitHub" },
      {
        time: "14:21:26",
        text: "Taken by a warm runner on",
        machine: "hetzner-ax42-1",
      },
      { time: "14:22:24", text: "Passed" },
    ],
    waited: "4s",
    duration: "58s",
    cache: { source: "machine", restored: 1, sizeGb: 0.4 },
  },
  "no-steps": {
    name: "unit (node 24)",
    context: `${repositoryName("mokly-cloud")} · CI · #318 Retry failed webhooks`,
    ref: "#318",
    where: own(3, 5),
    label: label.name,
    request: label,
    why: warmReason,
    timeline: [
      { time: "14:24:12", text: "Queued on GitHub" },
      {
        time: "14:24:12",
        text: "Taken by a warm runner on",
        machine: "hetzner-ax42-3",
      },
      { time: "14:26:06", text: "Passed" },
    ],
    waited: "0s",
    duration: "1m 54s",
    cache: { source: "machine", restored: 1, sizeGb: 0.5 },
  },
} as const satisfies Readonly<Record<string, JobDetail>>;
