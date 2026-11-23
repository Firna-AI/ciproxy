import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OverflowScreen } from "../../../../../screens/settings/overflow.tsx";

const address = orgAddress("settings/overflow");
const docs = related("overflow", "overflowSpend", "labels", "jobRouting");

const rationale =
  "The user's decisions of 23 November 2026: a workspace can connect several providers, each with one active key, its own Change key and Disconnect, and its own monthly spend limit. A provider that the workspace connects for the first time goes last in the order of each Linux label. Second provider is a fixture name, because this change ships no second real provider.";

export const connected = defineScreen({
  slug: "connected",
  title: "Two connected providers",
  description:
    "A what-if: mokly-ai connected Second provider on 9 November, beside Boat. Each provider has its own panel with its account, Change key, Disconnect, and its own monthly spend limit: $200 on Boat and $100 on Second provider. Second provider took the overflow of ciproxy-large since then, so its $17.40 of November is a part of the spend that Boat has in the main story. The switch and the wait above them hold for both providers.",
  rationale,
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="two-providers" />
  ),
});

export const connectSecond = defineScreen({
  slug: "connect-second",
  title: "Connect a second provider",
  description:
    "Boat is connected. Connect Second provider opens its form in its panel: its API key and the monthly spend limit on Second provider. The line above the form says that it goes last in the order of each Linux label, and that each label can move or remove it. It has no wallets, so Connect Second provider checks the key and connects in one step. Cancel keeps the page as it was.",
  rationale,
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="connect-second" />
  ),
});

export const secondLimitReached = defineScreen({
  slug: "second-limit-reached",
  title: "One provider reached its limit",
  description:
    "A what-if with two providers: the spend on Second provider reached its $100 limit for November. ciproxy starts no new jobs on Second provider until 1 December, or until the limit rises, and the labels go on to the next place of their order. Boat keeps its own limit and takes jobs as before.",
  rationale,
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="second-limit-reached" />
  ),
});

export const boatUnavailable = defineScreen({
  slug: "boat-unavailable",
  title: "A connected provider is unavailable",
  description:
    "A what-if with two providers: the deployment lost the settings of Boat, so Boat is unavailable right now. The notice says that ciproxy starts no new jobs on Boat until it is available again, and that jobs go on to the next place of their label. The Boat panel keeps the account mokly, Disconnect, the limit, and the spend of the month. No new connection can start, so Change key leaves. Second provider takes jobs as before. The label order shows the same moment in the row of Boat.",
  rationale: `${rationale} The plan says that Setup and Settings show a provider without its deployment settings as unavailable right now.`,
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="boat-unavailable" />
  ),
});
