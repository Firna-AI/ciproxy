import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../components/related-docs.ts";
import { host, orgAddress } from "../../../fixtures/org.ts";
import { AppRemovedScreen } from "../../../screens/auth/app-removed.tsx";
import { PersonalAccountScreen } from "../../../screens/auth/personal-account.tsx";

export const removed = defineScreen({
  slug: "removed",
  title: "The App was removed",
  description:
    "A what-if in which mokly-ai is the only organization of its workspace. Tom Becker, an owner of mokly-ai, removed the ciproxy GitHub App on GitHub on 20 November. Every page of the workspace opens this page for each person with a role. ciproxy revoked the machines at once, deletes the Boat key when the last running sandbox ends, and Team ends at the end of its period, on 3 December. A new install before 20 December finds the labels, the settings, and the job history; each machine needs the install command again, and Boat a new connection. Install on GitHub opens the install page of GitHub.",
  rationale:
    "After a removal, a GitHub user token probably cannot read the organization, so ciproxy cannot check who belongs to it. The page shows no data of mokly-ai, only that the App was removed and how to install it again. The suspended App is in Jobs, under Why jobs wait, because a suspension stops only the jobs.",
  relatedDocs: relatedMvp("appRemoval", "billing", "security"),
  address: orgAddress("jobs"),
  render: (_props, context) => <AppRemovedScreen viewport={context.viewport} />,
});

export const personalAccount = defineScreen({
  slug: "personal-account",
  title: "The App on a personal account",
  description:
    "Cal Moore installed the ciproxy GitHub App on calummoore, his personal account, so GitHub returns to this page instead of setup. Install on an organization opens the install page of GitHub, and Remove from calummoore opens the GitHub settings of the install.",
  rationale:
    "Design call 36: ciproxy serves organizations only, and a GitHub organization costs nothing, so the page leads to an organization and links to the removal of the App.",
  relatedDocs: relatedMvp("security"),
  address: `${host}/setup`,
  render: (_props, context) => (
    <PersonalAccountScreen viewport={context.viewport} />
  ),
});
