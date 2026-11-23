// The provider states that the browser checks for layout at each width, with
// Boat as the provider: each opens its page, shows its one fact, and keeps
// every control inside the viewport. Their copy and rules are Node tests;
// the journeys that act are in browser-boat.ts.
import { overflowHref, workspaceHref } from "../src/routing/pages.ts";
import type { SmokeCase } from "./browser-cases.ts";
import { workspaceId } from "./fixtures-core.ts";

const setup = workspaceHref("setup", workspaceId);
const overflow = overflowHref(workspaceId);

export const boatCases: readonly SmokeCase[] = [
  {
    name: "boat-setup-rejected",
    scenario: "boat-setup-rejected",
    path: setup,
    heading: "Set up ciproxy",
    text: "Boat rejected the API key on 12 November",
  },
  {
    name: "boat-setup-unavailable",
    scenario: "boat-setup-unavailable",
    path: setup,
    heading: "Set up ciproxy",
    text: "No overflow provider is available right now",
  },
  {
    name: "boat-setup-load-failure",
    scenario: "boat-setup-load-failure",
    path: setup,
    heading: "Set up ciproxy",
    text: "Unable to load the overflow providers",
  },
  {
    name: "boat-setup-job-running",
    scenario: "boat-setup-job-running",
    path: setup,
    heading: "Set up ciproxy",
    text: "Build and test in northstar-labs/control is running on your machine…",
  },
  {
    name: "overflow-rejected",
    scenario: "overflow-rejected",
    path: overflow,
    heading: "Settings",
    text: "Boat rejected the API key on 12 November",
  },
  {
    name: "overflow-limit-reached",
    scenario: "overflow-limit-reached",
    path: overflow,
    heading: "Settings",
    text: "Boat spend reached the $200 limit for November",
  },
  {
    name: "overflow-unknown-costs",
    scenario: "overflow-unknown-costs",
    path: overflow,
    heading: "Settings",
    text: "Boat costs are not known yet",
  },
  {
    name: "overflow-spend-unavailable",
    scenario: "overflow-spend-unavailable",
    path: overflow,
    heading: "Settings",
    text: "Unable to read the Boat spend",
  },
  {
    name: "overflow-unavailable",
    scenario: "overflow-unavailable",
    path: overflow,
    heading: "Settings",
    text: "No overflow provider is available right now",
  },
  {
    name: "overflow-connected-unavailable",
    scenario: "overflow-connected-unavailable",
    path: overflow,
    heading: "Settings",
    text: "Boat is unavailable right now",
  },
  {
    name: "overflow-load-failure",
    scenario: "overflow-load-failure",
    path: overflow,
    heading: "Settings",
    text: "Unable to load Overflow",
  },
];
