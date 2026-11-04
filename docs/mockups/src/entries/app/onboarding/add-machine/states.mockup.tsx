import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { planDoc } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../screens/onboarding/onboarding.tsx";

export const connected = defineScreen({
  slug: "connected",
  title: "The machine connected",
  description:
    "The agent connected and reported the machine: 16 threads, 64 GB, KVM ready, and 6 slots. Continue opens the Boat step.",
  relatedDocs: [planDoc],
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
  relatedDocs: [planDoc],
  address: orgAddress("setup"),
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="machine-no-kvm" />
  ),
});

export const noRunnerRight = defineScreen({
  slug: "no-runner-right",
  title: "The Admin cannot add machines",
  description:
    "Lena Fischer asked for the install, so she is the first Admin, but GitHub does not let her add self-hosted runners. She sees no install command, and an owner adds the machine or gives her the right. She can set up everything else.",
  rationale:
    "A machine gets the job secrets of the repositories, so adding one needs the Admin role and the runner right of the organization.",
  relatedDocs: [planDoc],
  address: orgAddress("setup"),
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="machine-no-right" />
  ),
});
