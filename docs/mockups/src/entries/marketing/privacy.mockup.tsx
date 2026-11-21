import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../components/related-docs.ts";
import { siteAddress } from "../../fixtures/site.ts";
import { LegalPageScreen } from "../../screens/marketing/legal-page.tsx";

export const privacy = defineScreen({
  title: "Privacy policy",
  description:
    "The frame of the privacy policy of ciproxy.dev, with its section headings. Data retention states the limits that are decided: 90 days of job details on every plan and of the daily totals, 7 days of cache hits, the last report of each machine, a person only while the person has a role or a request, and the data of an organization for 30 days after it removes the App.",
  rationale:
    "The user writes or approves the text of the other sections before the launch, so grey lines hold their place. Milestone 5 sets the periods of the logs and backups on GCP, which the policy also states once they are set.",
  relatedDocs: relatedMvp("security", "appRemoval"),
  address: siteAddress("privacy"),
  render: (_props, context) => (
    <LegalPageScreen viewport={context.viewport} page="privacy" />
  ),
});
