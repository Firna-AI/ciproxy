import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { planDoc } from "../../components/state-schema.ts";

export const firstGreenJob = defineUseCase({
  title: "From sign-in to the first green job",
  description:
    "Cal Moore signs in on 1 September 2026, installs the GitHub App on mokly-ai, adds one Hetzner machine with one command, connects Boat, and sees the first job pass on the own machine.",
  relatedDocs: [planDoc],
  steps: [
    {
      screenPath: paths.signIn,
      title: "Sign in with GitHub",
      description:
        "ciproxy has no password of its own; GitHub signs the person in.",
    },
    {
      screenPath: paths.installApp,
      title: "Install the GitHub App",
      description:
        "GitHub asks for the organization and its repositories, then returns to ciproxy setup.",
    },
    {
      screenPath: paths.onboarding,
      title: "Copy the install command",
      description:
        "Setup opens on the machine step. Cal runs the one command on hetzner-ax42-1 and the page waits.",
    },
    {
      screenPath: paths.machineConnected,
      title: "The machine connects",
      description: "The agent reports 16 threads, 64 GB, KVM, and 6 slots.",
    },
    {
      screenPath: paths.connectBoat,
      title: "Connect Boat",
      description:
        "Cal pastes an API key of the mokly Boat account for overflow.",
    },
    {
      screenPath: paths.firstJob,
      title: "Change runs-on",
      description: "One line in one workflow changes, and Cal pushes it.",
    },
    {
      screenPath: paths.firstJobPassed,
      title: "The first job passes",
      description:
        "A warm runner on hetzner-ax42-1 took the job at once. Setup is complete; Open Jobs leads to the live queue.",
    },
  ],
});
