import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OverflowScreen } from "../../../../../screens/settings/overflow.tsx";

const address = orgAddress("settings/overflow");
const docs = related("overflow", "security");

export const notConnected = defineScreen({
  slug: "not-connected",
  title: "Boat is not connected",
  description:
    "An organization that skipped the Boat step of setup. Connecting Boat here asks for the key and the monthly spend limit, as setup does, so a limit applies from the first job on Boat. After a disconnect, the limit field starts with the saved limit.",
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
    "Boat stopped accepting the API key on 12 November. Jobs run on own machines only until a new key connects. The reconnect asks for the new key and the spend limit, and the limit starts with the saved $200.",
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
    "Change key opens a new connection, which asks for the new key and the spend limit, with the saved $200. New Boat jobs start with the new key, and the jobs that run on Boat finish with the current key.",
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
    "Disconnect asks first. ciproxy then starts no new Boat jobs, the jobs that run on Boat finish, and ciproxy deletes the key when the last of their sandboxes ends.",
  relatedDocs: docs,
  address,
  useCasePaths: [paths.flowChangeBoatKey],
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="disconnect" />
  ),
});

export const unavailable = defineScreen({
  slug: "unavailable",
  title: "Boat is unavailable",
  description:
    "The deployment runs without the Boat service, so no connection can start, and jobs run on own machines only. Try again reads the settings again.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="unavailable" />
  ),
});
