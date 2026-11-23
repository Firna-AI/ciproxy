import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../screens/onboarding/onboarding.tsx";

export const connectProvider = defineScreen({
  title: "Connect an overflow provider",
  description:
    "The third step. The customer picks the overflow provider where ciproxy starts jobs when the machines are busy. Boat and Second provider, the fixture name of a later provider, are available in the deployment. Boat is picked, so the Boat form shows: an API key of the customer's own Boat account and the monthly spend limit on Boat. Check key asks Boat for the wallets of the key, which show under Boat branch. Skip for now continues without overflow. The other states of the step are under Provider step states.",
  rationale:
    "The user's decisions of 23 November 2026: a workspace can connect several providers, and each provider has its own monthly spend limit, so each connection asks for the limit of its provider. A provider that the workspace connects for the first time goes last in the order of each Linux label. The wallet choice is Boat's, so only the Boat branch asks for a wallet.",
  relatedDocs: related("overflow", "overflowSpend", "security"),
  address: orgAddress("setup"),
  useCasePaths: [paths.flowFirstGreenJob, paths.flowSkipBoat],
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="provider" />
  ),
});
