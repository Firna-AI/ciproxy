import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { planDoc } from "../../../components/state-schema.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../screens/onboarding/onboarding.tsx";

export const onboarding = defineScreen({
  title: "Setup",
  description:
    "The first page after the GitHub App is installed. Setup has four steps; here the second is open, with the one install command, and the page waits for the machine to connect.",
  rationale:
    "One command installs the agent and connects the server, so a person copies one line and needs no other step on the server.",
  relatedDocs: [planDoc],
  address: orgAddress("setup"),
  useCasePaths: [paths.flowFirstGreenJob],
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="machine-waiting" />
  ),
});
