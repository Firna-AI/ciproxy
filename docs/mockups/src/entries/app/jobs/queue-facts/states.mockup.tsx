import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../../screens/jobs/jobs-live.tsx";

const address = orgAddress("jobs");

export const boatNotConnected = defineScreen({
  slug: "boat-not-connected",
  title: "Boat is not connected",
  description:
    "The workspace has no Boat account. Every own slot runs a job and six jobs wait for a slot. Connect Boat opens the overflow settings.",
  relatedDocs: related("jobRouting", "overflow"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="boat-not-connected" />
  ),
});

export const appRemoved = defineScreen({
  slug: "app-removed",
  title: "The App was removed from an organization",
  description:
    "An owner removed the ciproxy GitHub App from mokly-labs. The jobs of mokly-ai keep running. The notice names mokly-labs, and Install the App opens the install on GitHub, which the API gives.",
  rationale:
    "app-removal.md: the dashboard says that the App was removed and how to install it again. The API gives no date of the removal, so the notice names none.",
  relatedDocs: related("appRemoval", "workspaces"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="app-removed" />
  ),
});

export const quietNoWarm = defineScreen({
  slug: "quiet-no-warm",
  title: "Nothing runs and no warm runner waits",
  description:
    "No job runs or waits, and no warm runner is ready, for example while a warm runner count is 0. The page claims no ready runner.",
  relatedDocs: related("jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="quiet-no-warm" />
  ),
});
