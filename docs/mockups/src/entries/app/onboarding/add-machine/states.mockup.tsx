import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../screens/onboarding/onboarding.tsx";

export const connected = defineScreen({
  slug: "connected",
  title: "The machine connected",
  description:
    "The agent connected and reported the machine: 16 threads, 64 GB, KVM ready, and 6 slots. Continue opens the Boat step.",
  relatedDocs: related("agent", "jobRouting", "security"),
  address: orgAddress("setup"),
  useCasePaths: [paths.flowFirstGreenJob],
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="machine-connected" />
  ),
});

export const noKvm = defineScreen({
  slug: "no-kvm",
  title: "The machine cannot start VMs",
  description:
    "The agent connected, but the machine has no KVM, so it cannot start a VM for each job. The notice names the fix; the command stays on the page to run again.",
  relatedDocs: related("agent", "jobRouting", "security"),
  address: orgAddress("setup"),
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="machine-no-kvm" />
  ),
});

export const noRunnerRight = defineScreen({
  slug: "no-runner-right",
  title: "The Admin cannot add machines",
  description:
    "Lena Fischer asked for the install, so she is the first Admin, but GitHub does not let her manage self-hosted runners. In place of the install command, she sees who can add the machine: an owner of the organization, or an Admin with the GitHub runner right. She can set up everything else, and Machines and Add machine show the same notice.",
  rationale:
    "A machine gets the job secrets of the repositories, so adding one needs the Admin role and the runner right of the organization.",
  relatedDocs: related("agent", "jobRouting", "security"),
  address: orgAddress("setup"),
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="machine-no-right" />
  ),
});
