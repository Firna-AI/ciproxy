import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { related, relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../../screens/onboarding/onboarding.tsx";

const address = orgAddress("setup");
const docs = related("overflow", "security");

export const chooseWallet = defineScreen({
  slug: "choose-wallet",
  title: "Choose the wallet",
  description:
    "Boat accepted the key. The form lists the wallets that the key can use: Cal Moore's personal wallet and the wallets of the Boat organizations mokly and driftwood. The key check names mokly as likely, so it starts picked, and Cal can pick another wallet. Connect Boat then connects the key, the wallet, and the limit. A change of the key clears the wallets, and Check key runs again.",
  rationale:
    "The owner's answer B to item 94, 8 November 2026: after the key check, the form lists the personal wallet and each Boat organization wallet, with the likely one preselected. Each Boat organization has its own wallet. Wallets are a Boat thing, so only the Boat branch shows them.",
  relatedDocs: relatedMvp("overflow", "security"),
  address,
  useCasePaths: [paths.flowFirstGreenJob],
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="boat-wallet" />
  ),
});

export const invalidKey = defineScreen({
  slug: "invalid-key",
  title: "Boat rejected the key",
  description:
    "Check key found that Boat did not accept the key, so no wallet shows and ciproxy saved nothing. The field error says how to get a complete key; the spend limit keeps its value.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="boat-invalid" />
  ),
});

export const connectFailed = defineScreen({
  slug: "connect-failed",
  title: "The connection did not finish",
  description:
    "After the key check and the wallet choice, ciproxy could not reach Boat, so nothing changed. The fields and the picked wallet keep their values, the key masked, and Connect Boat tries again.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="boat-connect-failed" />
  ),
});

export const rejected = defineScreen({
  slug: "rejected",
  title: "Boat rejected the saved key",
  description:
    "Boat connected earlier, and on 12 November it stopped accepting the key. The step asks for a new Boat key, with no provider choice, and the limit starts with the saved $200.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="boat-rejected" />
  ),
});
