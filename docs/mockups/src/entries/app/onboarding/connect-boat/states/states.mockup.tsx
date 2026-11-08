import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../../screens/onboarding/onboarding.tsx";

const address = orgAddress("setup");
const docs = related("overflow", "security");

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
    "Boat connected earlier, and on 12 November it stopped accepting the key. The step asks for a new key, and the limit starts with the saved $200.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="boat-rejected" />
  ),
});

export const unavailable = defineScreen({
  slug: "unavailable",
  title: "Boat is unavailable",
  description:
    "The deployment runs without the Boat service, so no key can connect now. Skip for now continues to the first job, and Settings under Overflow connects Boat later.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="boat-unavailable" />
  ),
});

export const loadFailed = defineScreen({
  slug: "load-failed",
  title: "The Boat step did not load",
  description:
    "The Boat settings did not load. Try again reads them again; Skip for now continues to the first job.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="boat-load-failed" />
  ),
});
