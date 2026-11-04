import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { SavingsScreen } from "../../../../screens/savings/savings.tsx";

export const viewer = defineScreen({
  slug: "viewer",
  title: "Savings, seen by a Viewer or a Billing person",
  description:
    "Jonas Weber, a Viewer, sees the report read-only. No machine has a monthly cost yet, and in place of Add costs he reads Ask an Admin. A Billing person sees the same page, because only an Admin enters what a machine costs.",
  rationale:
    "Billing adds the plan, the add-ons, and the invoices to what a Viewer sees, and the report has none of them to act on, so one screen serves both roles.",
  relatedDocs: related("metering", "security"),
  address: orgAddress("savings"),
  render: (_props, context) => (
    <SavingsScreen
      viewport={context.viewport}
      state="missing-costs"
      role="viewer"
    />
  ),
});
