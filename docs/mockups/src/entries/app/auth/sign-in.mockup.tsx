import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { planDoc, stateSchema } from "../../../components/state-schema.ts";
import { host } from "../../../fixtures/org.ts";
import { SignInScreen } from "../../../screens/auth/sign-in.tsx";

export const signIn = defineScreen({
  title: "Sign in",
  description:
    "The first screen of ciproxy. A person signs in with GitHub; ciproxy has no password of its own.",
  relatedDocs: [planDoc],
  address: `${host}/sign-in`,
  useCasePaths: [paths.flowFirstGreenJob],
  propSchema: stateSchema(["ready", "failed"]),
  props: { state: "ready" },
  render: (props, context) => (
    <SignInScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "failed",
      title: "Sign-in did not finish",
      description:
        "GitHub returned without a signed-in user, for example after Cancel on the GitHub page. The same button signs in again.",
      props: { state: "failed" },
    },
  ],
});
