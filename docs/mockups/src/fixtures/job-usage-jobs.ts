// The CPU and memory of the jobs of the job page, from the profile of each
// step: its name as the runner's log gives it, its seconds, its CPU level in
// vCPU, and its memory ramp in GB. The steps of each job last as long as the
// job does. A step that GitHub names from a run command shows as Run script,
// and so does the step that eval (agents) names Run evals, because its name
// starts with "Run " and names no remote action. The step that test (rust)
// names Running tests keeps its name. A post step of under a second holds no
// point. On Boat, the figures run against the Boat size that ran the job,
// and they name the size of the label too. visual diff ran out of memory in
// Compare screenshots: the kernel stopped one process, the step failed, and
// the job went on. The pages of the current API show the same figures as
// their jobs, because the job API gives them. A job that no runner took has
// no panel.

import { jobDetails, type JobDetailState } from "./job-details.ts";
import { measured, type JobUsage } from "./job-usage.ts";

/** test (rust) on hetzner-ax42-1, also as the current API shows it. */
const own = measured(jobDetails.own, [
  ["Set up job", 2, 0.3, 0.42, 0.48],
  ["Run actions/checkout@v4", 2, 0.35, 0.5, 0.55],
  ["Run dtolnay/rust-toolchain@stable", 8, 0.9, 0.55, 0.85],
  ["Run Swatinem/rust-cache@v2", 13, 1.2, 0.9, 1.45],
  ["Run cargo fetch --locked", 9, 0.55, 1.4, 1.3],
  ["Build", 166, 1.95, 2.0, 4.9],
  ["Running tests", 160, 1.55, 3.4, 6.1],
  ["Post Run Swatinem/rust-cache@v2", 11, 0.95, 1.9, 1.7],
  ["Post Run actions/checkout@v4", 0],
  ["Complete job", 1, 0.2, 1.0, 1.0],
]);

/** e2e (chromium) 3/4 on Boat, also as the current API shows it. */
const boat = measured(jobDetails.boat, [
  ["Set up job", 3, 0.6, 0.7, 0.75],
  ["Run actions/checkout@v4", 2, 0.5, 0.75, 0.8],
  ["Run actions/setup-node@v4", 6, 1.1, 0.8, 1.0],
  ["Run npm ci", 38, 2.6, 1.1, 2.3],
  ["Install Playwright browsers", 41, 1.8, 2.3, 2.7],
  ["Build", 92, 6.4, 2.8, 4.9],
  ["Test", 239, 4.6, 4.2, 7.4],
  ["Post Run actions/setup-node@v4", 3, 0.9, 1.9, 1.8],
  ["Post Run actions/checkout@v4", 0],
  ["Complete job", 1, 0.3, 1.2, 1.2],
]);

export const jobUsage: Readonly<Partial<Record<JobDetailState, JobUsage>>> = {
  own,
  boat,
  "current-own": own,
  "current-boat": boat,
  fallback: measured(jobDetails.fallback, [
    ["Set up job", 2, 0.3, 0.42, 0.48],
    ["Run actions/checkout@v4", 2, 0.35, 0.5, 0.55],
    ["Run dtolnay/rust-toolchain@stable", 8, 0.9, 0.55, 0.85],
    ["Run Swatinem/rust-cache@v2", 34, 1.0, 0.9, 1.5],
    ["Run cargo clippy --all-targets -- -D warnings", 109, 1.9, 1.7, 3.6],
    ["Post Run Swatinem/rust-cache@v2", 12, 0.9, 1.8, 1.6],
    ["Post Run actions/checkout@v4", 0],
    ["Complete job", 1, 0.2, 0.9, 0.9],
  ]),
  "mixed-cache": measured(jobDetails["mixed-cache"], [
    ["Set up job", 2, 0.3, 0.42, 0.48],
    ["Run actions/checkout@v4", 2, 0.35, 0.5, 0.55],
    ["Run actions/setup-python@v5", 5, 0.8, 0.6, 0.8],
    ["Run pip install -r requirements.txt", 41, 1.1, 0.9, 1.9],
    ["Run evals", 252, 1.2, 2.0, 3.9],
    ["Post Run actions/setup-python@v5", 5, 0.8, 1.6, 1.5],
    ["Post Run actions/checkout@v4", 0],
    ["Complete job", 1, 0.2, 0.9, 0.9],
  ]),
  "no-restores": measured(jobDetails["no-restores"], [
    ["Set up job", 2, 0.3, 0.42, 0.48],
    ["Run actions/checkout@v4", 1, 0.35, 0.5, 0.5],
    ["Run dtolnay/rust-toolchain@stable", 11, 0.8, 0.5, 0.75],
    ["Run cargo fmt --all --check", 5, 1.4, 0.75, 0.8],
    ["Post Run actions/checkout@v4", 0],
    ["Complete job", 2, 0.2, 0.6, 0.6],
  ]),
  "unknown-request": measured(jobDetails["unknown-request"], [
    ["Set up job", 2, 0.3, 0.42, 0.48],
    ["Run actions/checkout@v4", 2, 0.35, 0.5, 0.55],
    ["Run actions/setup-node@v4", 4, 0.8, 0.6, 0.8],
    ["Run npm ci", 33, 1.3, 0.9, 2.0],
    ["Build Storybook", 158, 1.85, 2.2, 5.3],
    ["Upload Storybook", 10, 0.6, 2.4, 2.2],
    ["Post Run actions/setup-node@v4", 3, 0.8, 1.9, 1.8],
    ["Post Run actions/checkout@v4", 0],
    ["Complete job", 1, 0.2, 1.0, 1.0],
  ]),
  "unknown-request-boat": measured(jobDetails["unknown-request-boat"], [
    ["Set up job", 3, 0.6, 0.7, 0.75],
    ["Run actions/checkout@v4", 2, 0.5, 0.75, 0.8],
    ["Run docker/setup-buildx-action@v3", 7, 0.9, 0.9, 1.3],
    ["Run docker/login-action@v3", 1, 0.4, 1.3, 1.3],
    ["Build and push", 262, 5.8, 2.0, 6.9],
    ["Post Run docker/setup-buildx-action@v3", 4, 0.7, 2.1, 1.9],
    ["Post Run actions/checkout@v4", 0],
    ["Complete job", 1, 0.3, 1.2, 1.2],
  ]),
  "unknown-facts": { state: "unavailable" },
  "out-of-memory": measured(
    jobDetails["out-of-memory"],
    [
      ["Set up job", 2, 0.3, 0.42, 0.48],
      ["Run actions/checkout@v4", 2, 0.35, 0.5, 0.55],
      ["Run actions/setup-node@v4", 5, 0.8, 0.6, 0.8],
      ["Run npm ci", 29, 1.3, 0.9, 1.9],
      ["Build", 58, 1.9, 2.0, 3.6],
      ["Compare screenshots", 91, 1.75, 3.4, 7.9],
      ["Upload diff images", 3, 0.5, 1.7, 1.6],
      ["Post Run actions/setup-node@v4"],
      ["Post Run actions/checkout@v4", 1, 0.2, 1.4, 1.4],
      ["Complete job", 1, 0.2, 1.2, 1.2],
    ],
    { kill: { second: 174, cpu: 0.5, memoryGb: 2.4 } },
  ),
  macos: measured(
    jobDetails.macos,
    [
      ["Set up job", 2, 0.4, 2.1, 2.15],
      ["Run actions/checkout@v4", 3, 0.5, 2.2, 2.25],
      ["Run actions/setup-node@v4", 5, 0.9, 2.3, 2.5],
      ["Run npm ci", 46, 1.4, 2.5, 3.3],
      ["Build", 118, 1.85, 3.4, 5.2],
      ["Test", 288, 1.45, 4.6, 6.8],
      ["Post Run actions/setup-node@v4", 4, 0.8, 3.1, 3.0],
      ["Post Run actions/checkout@v4", 1, 0.3, 2.8, 2.8],
      ["Complete job", 1, 0.3, 2.6, 2.6],
    ],
    { platform: "macos" },
  ),
  running: { state: "running" },
  "no-figures": { state: "unavailable" },
  "no-steps": measured(
    jobDetails["no-steps"],
    [
      ["Set up job", 2, 0.3, 0.42, 0.48],
      ["Run actions/checkout@v4", 2, 0.35, 0.5, 0.55],
      ["Run actions/setup-node@v4", 4, 0.8, 0.6, 0.8],
      ["Run npm ci", 24, 1.3, 0.9, 1.7],
      ["Test", 80, 1.6, 1.8, 3.2],
      ["Post Run actions/checkout@v4", 1, 0.2, 1.2, 1.2],
      ["Complete job", 1, 0.2, 1.0, 1.0],
    ],
    { noSteps: true },
  ),
};
