import { defineScreen } from "@mokly/mokly";

import { related, relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OverflowScreen } from "../../../../../screens/settings/overflow.tsx";

const address = orgAddress("settings/overflow");
const docs = related("overflow", "jobRouting");

export const limitReached = defineScreen({
  slug: "limit-reached",
  title: "The spend limit is reached",
  description:
    "The confirmed and the unconfirmed costs together reached the limit. Jobs run on own machines only until the month ends or the limit rises. The title names the budget month, November, as the Jobs notice does. The date when Boat takes jobs again is the first day of the next month in UTC, and the budget month is the UTC month before it.",
  rationale:
    "The user's answer C to item 109, 8 November 2026: the notice names the budget month and the day when Boat takes jobs again. Both come from the API date of the next month, and the API gives no day on which the spend reached the limit.",
  relatedDocs: relatedMvp("overflow", "jobRouting"),
  address,
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="limit-reached" />
  ),
});

export const unknownCosts = defineScreen({
  slug: "unknown-costs",
  title: "A cost is not known yet",
  description:
    "A start has no safe bound yet, so new Boat starts wait until Boat reports its cost. The figures show the costs that ciproxy knows, and the page claims no reached limit.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="unknown-costs" />
  ),
});

export const spendUnavailable = defineScreen({
  slug: "spend-unavailable",
  title: "The spend cannot be read",
  description:
    "ciproxy could not read the costs of the month from Boat. The limit still applies and can change; Try again reads the spend again.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="spend-unavailable" />
  ),
});

export const saveFailed = defineScreen({
  slug: "save-failed",
  title: "The limit did not save",
  description:
    "Save did not finish. The field keeps the typed $300, and the saved limit stays $200 until Save succeeds.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="save-failed" />
  ),
});
