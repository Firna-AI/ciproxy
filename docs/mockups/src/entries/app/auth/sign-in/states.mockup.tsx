import { defineScreen } from "@mokly/mokly";

import { related, relatedMvp } from "../../../../components/related-docs.ts";
import { host } from "../../../../fixtures/org.ts";
import { SignInScreen } from "../../../../screens/auth/sign-in.tsx";

const address = `${host}/sign-in`;

export const failed = defineScreen({
  slug: "failed",
  title: "Sign-in did not finish",
  description:
    "GitHub returned without a signed-in user, for example after Cancel on the GitHub page. The same button signs in again.",
  relatedDocs: related("security"),
  address,
  render: (_props, context) => (
    <SignInScreen viewport={context.viewport} state="failed" />
  ),
});

export const expired = defineScreen({
  slug: "expired",
  title: "A sign-in ended after 30 days",
  description:
    "A sign-in lasts 30 days. After that, every page of the dashboard opens sign-in with a notice that says why, and the same button signs in again.",
  rationale:
    "Design call 35: a sign-in lasts 30 days, and ciproxy checks the membership of the person each hour, so access ends soon after a person leaves the organization.",
  relatedDocs: relatedMvp("security"),
  address,
  render: (_props, context) => (
    <SignInScreen viewport={context.viewport} state="expired" />
  ),
});
