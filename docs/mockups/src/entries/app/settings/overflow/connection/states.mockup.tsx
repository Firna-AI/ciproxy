import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OverflowScreen } from "../../../../../screens/settings/overflow.tsx";

const address = orgAddress("settings/overflow");
const docs = related("overflow", "overflowSpend", "security");

export const notConnected = defineScreen({
  slug: "not-connected",
  title: "No provider is connected",
  description:
    "An organization that skipped the provider step of setup, or disconnected its only provider. The page asks for a provider first, as setup does, with Boat picked, then for its key and its monthly spend limit, so a limit applies from the first job on the provider. Check key lists the wallets of the Boat key first. After a disconnect, the limit field starts with the saved limit of that provider.",
  relatedDocs: docs,
  address,
  useCasePaths: [paths.flowSkipBoat, paths.flowChangeBoatKey],
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="not-connected" />
  ),
});

export const boatRejected = defineScreen({
  slug: "boat-rejected",
  title: "Boat rejected the key",
  description:
    "Boat stopped accepting the API key on 12 November. ciproxy starts no new jobs on Boat until a new key connects, and the labels go on to their next place. The Boat panel asks for the new key and the spend limit on Boat, which starts with the saved $200. The account keeps Disconnect beside Check key, and it opens the same dialog.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="boat-rejected" />
  ),
});

export const changeKey = defineScreen({
  slug: "change-key",
  title: "Change the key",
  description:
    "Change key of Boat opens a new connection in the Boat panel, which asks for the new key and the spend limit on Boat, with the saved $200. Check key lists the wallets of the new key. New Boat jobs start with the new key, and the jobs that run on Boat finish with the current key.",
  relatedDocs: docs,
  address,
  useCasePaths: [paths.flowChangeBoatKey],
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="change-key" />
  ),
});

export const disconnect = defineScreen({
  slug: "disconnect",
  title: "Disconnect Boat",
  description:
    "Disconnect asks first. ciproxy then starts no new Boat jobs, the jobs that run on Boat finish, and ciproxy deletes the key when the last of their sandboxes ends. The labels keep Boat in their order and skip it, so a later connection of Boat needs no label edit. An account whose key Boat rejected gets the same dialog.",
  relatedDocs: docs,
  address,
  useCasePaths: [paths.flowChangeBoatKey],
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="disconnect" />
  ),
});

export const unavailable = defineScreen({
  slug: "unavailable",
  title: "No provider is available",
  description:
    "The deployment runs without the settings of every provider, so no connection can start, and jobs run on own machines only. Try again reads the settings again. When only some providers are unavailable, the provider choice shows each of them as unavailable right now, as setup does.",
  relatedDocs: related("overflow", "security"),
  address,
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="unavailable" />
  ),
});
