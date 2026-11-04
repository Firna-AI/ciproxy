import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OnboardingScreen } from "../../../../screens/onboarding/onboarding.tsx";

export const connectBoat = defineScreen({
  title: "Connect Boat",
  description:
    "The third step. The customer pastes an API key of its own Boat account, where ciproxy starts overflow jobs, and sets the monthly spend limit before Boat connects. Skip for now continues without overflow.",
  relatedDocs: related("overflow", "security"),
  address: orgAddress("setup"),
  useCasePaths: [paths.flowFirstGreenJob],
  propSchema: stateSchema(["boat", "boat-invalid"]),
  props: { state: "boat" },
  render: (props, context) => (
    <OnboardingScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "invalid-key",
      title: "Boat rejected the key",
      description:
        "Boat did not accept the key. The field error says how to get a complete key; the spend limit keeps its value.",
      props: { state: "boat-invalid" },
    },
  ],
});
