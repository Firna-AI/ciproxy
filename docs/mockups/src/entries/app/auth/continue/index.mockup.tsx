import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { host } from "../../../../fixtures/org.ts";
import { TermsStepScreen } from "../../../../screens/auth/terms-step.tsx";

export const termsStep = defineScreen({
  title: "The terms step",
  description:
    "The page before each GitHub sign-in that does not start on sign-in. GitHub returns here after a person installs the App on GitHub or asks the owners of an organization for the install. The sign-in links of the dashboard open it too: Sign in with GitHub of an install that ciproxy cannot confirm, and Check again of a sign-in with no workspace, as on the install page that waits for an owner and on the page of an ended access. The page shows the terms notice of sign-in. Continue with GitHub starts the GitHub sign-in, and the dashboard then opens the next page as before: Setup after a first install, the choice of a workspace after Add an organization, the page of a personal account, or the install page that waits for an owner. In the catalogue, Continue with GitHub opens Setup, the most common next page.",
  rationale:
    "The terms say that a person accepts the current terms at each sign-in with GitHub. The agent call of 22 November 2026, under the user's delegation: every GitHub sign-in that does not start on sign-in passes this page first, with the terms notice of sign-in from one shared part. The GitHub sign-in starts only when the person selects Continue with GitHub. The page is static: it names no install, no person, and no next page, so it shows in full before the dashboard reads anything.",
  relatedDocs: related("security", "workspaces"),
  address: `${host}/continue`,
  useCasePaths: [
    paths.flowFirstGreenJob,
    paths.flowAddOrganization,
    paths.flowChooseWorkspace,
    paths.flowApprovedLater,
    paths.flowFreeAddsOrganization,
    paths.flowRemovedOrganization,
  ],
  render: (_props, context) => <TermsStepScreen viewport={context.viewport} />,
});
