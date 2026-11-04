import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { relatedMvp } from "../../../components/related-docs.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../screens/onboarding/onboarding.tsx";

export const onboarding = defineScreen({
  title: "Setup",
  description:
    "The first page after the GitHub App is installed. Setup has four steps; here the second is open, with the one install command, and the page waits for the machine to connect.",
  rationale:
    "One command installs the agent and connects the machine, so a person copies one line and needs no other step on the machine. Design call 30: the install script of get.ciproxy.dev installs the agent, then runs it with the arguments after the two dashes, so the agent runs install with the token.",
  relatedDocs: relatedMvp("agent", "security"),
  address: orgAddress("setup"),
  useCasePaths: [paths.flowFirstGreenJob],
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="machine-waiting" />
  ),
});
