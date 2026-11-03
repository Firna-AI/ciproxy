import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { LabelsScreen } from "../../../../screens/settings/labels.tsx";

export const loading = defineScreen({
  slug: "loading",
  title: "While settings load",
  description:
    "Skeleton rows keep the shape of the labels table. The other tabs load the same way.",
  relatedDocs: [planDoc],
  address: orgAddress("settings/labels"),
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="loading" />
  ),
});

export const error = defineScreen({
  slug: "error",
  title: "When settings fail to load",
  description: "The shared failed-read state with Try again.",
  relatedDocs: [planDoc],
  address: orgAddress("settings/labels"),
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="error" />
  ),
});
