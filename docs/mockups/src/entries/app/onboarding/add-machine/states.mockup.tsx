import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related, relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../screens/onboarding/onboarding.tsx";

export const preparing = defineScreen({
  slug: "preparing",
  title: "The machine gets its job image",
  description:
    "The agent connected and reported the machine: 16 threads, 64 GB, KVM ready, and 6 slots. hetzner-ax42-1 gets its first job image, so it takes no job yet, and the line under its facts says that jobs start on it when it is ready. Continue opens the Boat step: Setup goes on during the download.",
  rationale:
    "The answer E to item 124: every page that shows a new machine says that it takes no job until it has its first job image. The machine did connect, so the check and the word connected stay. No text promises a time, because a slow link makes the download long.",
  relatedDocs: relatedMvp("agent", "jobRouting", "security"),
  address: orgAddress("setup"),
  useCasePaths: [paths.flowFirstGreenJob],
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="machine-preparing" />
  ),
});

export const connected = defineScreen({
  slug: "connected",
  title: "The machine connected",
  description:
    "The agent connected and reported the machine: 16 threads, 64 GB, KVM ready, and 6 slots. hetzner-ax42-1 has its job image, so it takes jobs. Continue opens the Boat step.",
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
