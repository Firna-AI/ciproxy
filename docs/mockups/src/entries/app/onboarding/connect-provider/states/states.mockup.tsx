import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../../screens/onboarding/onboarding.tsx";

const address = orgAddress("setup");
const docs = related("overflow", "security");

export const secondProvider = defineScreen({
  slug: "second-provider",
  title: "Second provider is picked",
  description:
    "Cal picks Second provider, the fixture name of a later provider. Its form asks for its API key and the monthly spend limit on Second provider. It has no wallets, so Connect Second provider checks the key and connects in one step. The plan of the real second provider sets its key check and its copy.",
  rationale:
    "The user's decisions of 23 November 2026: several providers, one spend limit for each provider, and no second real provider in this change, so a fixture name stands in for it.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OnboardingScreen viewport={context.viewport} state="second-provider" />
  ),
});

export const unavailable = defineScreen({
  slug: "unavailable",
  title: "Boat is unavailable",
  description:
    "The deployment runs without the Boat settings, so Boat shows as unavailable right now and cannot be picked. Second provider is available and picked, with its form. Skip for now continues to the first job.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OnboardingScreen
      viewport={context.viewport}
      state="provider-unavailable"
    />
  ),
});

export const noneAvailable = defineScreen({
  slug: "none-available",
  title: "No provider is available",
  description:
    "The deployment runs without the settings of every provider, so no key can connect now. Skip for now continues to the first job, and Settings under Overflow connects a provider later.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OnboardingScreen
      viewport={context.viewport}
      state="no-provider-available"
    />
  ),
});

export const loadFailed = defineScreen({
  slug: "load-failed",
  title: "The provider step did not load",
  description:
    "The overflow providers did not load. Try again reads them again; Skip for now continues to the first job.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OnboardingScreen
      viewport={context.viewport}
      state="provider-load-failed"
    />
  ),
});
