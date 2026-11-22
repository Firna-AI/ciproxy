import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { related } from "../../../components/related-docs.ts";
import { host } from "../../../fixtures/org.ts";
import { SignInScreen } from "../../../screens/auth/sign-in.tsx";

export const signIn = defineScreen({
  title: "Sign in",
  description:
    "The first screen of ciproxy. A person signs in with GitHub; ciproxy has no password of its own. The footer shows the terms notice, which the terms step shows too.",
  relatedDocs: related("security"),
  address: `${host}/sign-in`,
  useCasePaths: [
    paths.flowFirstGreenJob,
    paths.flowLandingToSignIn,
    paths.flowMemberAccess,
  ],
  render: (_props, context) => (
    <SignInScreen viewport={context.viewport} state="ready" />
  ),
});
