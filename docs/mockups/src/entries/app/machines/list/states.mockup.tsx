import { defineScreen } from "@mokly/mokly";

import { related, relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { MachinesScreen } from "../../../../screens/machines/machines-list.tsx";

const address = orgAddress("machines");

export const empty = defineScreen({
  slug: "empty",
  title: "No machines yet",
  description:
    "An organization that skipped the machine step of setup. Add machine opens the install command.",
  relatedDocs: related("agent", "metering"),
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="empty" />
  ),
});

export const offline = defineScreen({
  slug: "offline",
  title: "A machine is offline",
  description:
    "ciproxy lost contact with hetzner-ax42-3. New jobs go to the other machines and to Boat.",
  relatedDocs: related("agent", "metering"),
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="offline" />
  ),
});

export const overLimit = defineScreen({
  slug: "over-limit",
  title: "More own machines than the tier includes",
  description:
    "office-mini-1, a mini PC with a small disk, joined as the fourth machine on Team, which includes three. All four run jobs; the banner on every page offers the extra-machine add-on. office-mini-1 has 6 slots, but its disk holds 3 jobs of ciproxy at a time; its row opens its machine page, which says why.",
  relatedDocs: related("agent", "metering"),
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="over-limit" />
  ),
});

export const cacheUnknown = defineScreen({
  slug: "cache-unknown",
  title: "A machine cache is unknown",
  description:
    "hetzner-ax42-1 did not measure its machine cache for its last report. Cache hits, 7 days covers the cache restores of hetzner-ax42-2 and hetzner-ax42-3, the machines that measured their cache, and the line under it names hetzner-ax42-1. On desktop, the Cache column of hetzner-ax42-1 says Unknown. Its row opens its machine page.",
  rationale:
    "The answer E to item 118: the share counts only the machines that measured their cache. It names each machine that it leaves out, also a machine with no report yet, so the share never hides one. The mobile list has no Cache column, so the line under the figure names the machine there too.",
  relatedDocs: relatedMvp("cache", "agent"),
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="cache-unknown" />
  ),
});

export const noCacheMeasured = defineScreen({
  slug: "no-cache-measured",
  title: "No machine measured its cache",
  description:
    "None of the three machines measured its machine cache for its last report. Cache hits, 7 days says Unknown, not a share and not No restores yet, which stays for measured machines with no restores. On desktop, each Cache cell says Unknown.",
  rationale:
    "The answer E to item 118: with no measured machine, the share of all machines is unknown.",
  relatedDocs: relatedMvp("cache", "agent"),
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="no-cache-measured" />
  ),
});
