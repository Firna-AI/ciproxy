import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../components/related-docs.ts";
import { siteAddress } from "../../fixtures/site.ts";
import { LegalPageScreen } from "../../screens/marketing/legal-page.tsx";

export const terms = defineScreen({
  title: "Terms of service",
  description:
    "The frame of the terms of ciproxy.dev: the title, the switch to the privacy policy, On this page on a wide desktop, and the section headings. The footer of every marketing page and the sign-in page link here.",
  rationale:
    "The user writes or approves the text of each section before the launch, so grey lines hold its place. The headings name what the terms must cover: the service, accounts, plans and payment, the machines and Boat of the customer, use, data, the open-source agent, the end of the service, liability, changes, and contact.",
  relatedDocs: relatedMvp("billing", "security", "appRemoval"),
  address: siteAddress("terms"),
  render: (_props, context) => (
    <LegalPageScreen viewport={context.viewport} page="terms" />
  ),
});
