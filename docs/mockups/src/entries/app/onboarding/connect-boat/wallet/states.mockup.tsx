import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../../screens/onboarding/onboarding.tsx";

export const chooseWallet = defineScreen({
  slug: "choose-wallet",
  title: "Choose the wallet",
  description:
    "Boat accepted the key. The form lists the wallets that the key can use: Cal Moore's personal wallet and the wallets of the Boat organizations mokly and driftwood. The key check names mokly as likely, so it starts picked, and Cal can pick another wallet. Connect Boat then connects the key, the wallet, and the limit. A change of the key clears the wallets, and Check key runs again.",
  rationale:
    "The owner's answer B to item 94, 8 November 2026: after the key check, the form lists the personal wallet and each Boat organization wallet, with the likely one preselected. Each Boat organization has its own wallet.",
  relatedDocs: relatedMvp("overflow", "security"),
  address: orgAddress("setup"),
  useCasePaths: [paths.flowFirstGreenJob],
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="boat-wallet" />
  ),
});
