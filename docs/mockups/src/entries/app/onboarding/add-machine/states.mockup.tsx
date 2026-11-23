import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related, relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../screens/onboarding/onboarding.tsx";

export const preparing = defineScreen({
  slug: "preparing",
  title: "The machine gets its job image",
  description:
    "The agent connected and reported the machine: 16 threads, 64 GB, KVM ready, and 6 slots. hetzner-ax42-1 gets its first job image, so it takes no job yet, and the line under its facts says that jobs start on it when it is ready. Continue opens the provider step: Setup goes on during the download.",
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
    "The agent connected and reported the machine: 16 threads, 64 GB, KVM ready, and 6 slots. hetzner-ax42-1 has its job image, so it takes jobs. Continue opens the provider step.",
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

export const mac = defineScreen({
  slug: "mac",
  title: "A Mac connected",
  description:
    "A what-if: the first machine of mokly-ai is mac-studio-1, a Mac Studio. The same command connected it. The line under its name gives its cores, memory, virtualization, and 6 slots, and the next line says that it runs the jobs of macOS labels, at most 2 at a time. The note under the command names what a Linux machine and a Mac need: on a Mac, Apple Silicon, macOS 26.7.3 or later, 16 GB of memory, and 512 GB of disk, of which the disk is a placeholder until the size of the macOS image is measured. Continue opens the provider step, which only Linux jobs use.",
  rationale:
    "The agent install doc says that the one install command detects macOS and installs the agent as a launchd service. A Mac uses the slot rule of a Linux machine and runs at most 2 job VMs at a time, because Apple allows at most 2 macOS VMs at a time on one Mac.",
  relatedDocs: related("agent", "agentInstall", "jobRouting"),
  address: orgAddress("setup"),
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="machine-mac" />
  ),
});
