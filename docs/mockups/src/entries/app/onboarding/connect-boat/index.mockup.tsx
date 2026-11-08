import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../screens/onboarding/onboarding.tsx";

export const connectBoat = defineScreen({
  title: "Connect Boat",
  description:
    "The third step. The customer pastes an API key of its own Boat account, where ciproxy starts overflow jobs, and sets the monthly spend limit before Boat connects. Check key asks Boat for the wallets of the key, which show under Boat wallet. Skip for now continues without overflow. The other states of the step are under Boat step states.",
  relatedDocs: related("overflow", "security"),
  address: orgAddress("setup"),
  useCasePaths: [paths.flowFirstGreenJob, paths.flowSkipBoat],
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="boat" />
  ),
});
