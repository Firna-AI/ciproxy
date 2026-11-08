// The banner of the billing pages as the current API words it. The API gives
// the dates of a failed payment and the offer that covers the use, but not
// the plan, the limit that passed, or the day on which a paid plan ended. So
// the banner of a passed limit and of an ended plan stays general. The
// product counts every billing page as Plan and usage, so there the banner of
// a passed limit compares the plans. On Plans itself, a link to Plans would
// only reload the page, so it opens Plan and usage.

import type { ReactNode } from "react";
import { ExternalLink } from "lucide-react";

import {
  apiOffer,
  choiceCapacity,
  choiceMonthly,
  choiceName,
  passedLimit,
} from "../fixtures/current-api.ts";
import { workspace } from "../fixtures/org.ts";
import { paymentDates, type UsageScene } from "../fixtures/usage.ts";
import { Action, LinkAction } from "./actions.tsx";
import { Notice } from "./feedback.tsx";
import type { CataloguePath } from "./paths.ts";

/** Where the banner of a passed limit or of an ended plan leads. */
type BannerLink =
  /** The Plans screen of the moment, from each billing page but Plans. */
  | { plans: CataloguePath }
  /** On Plans itself: Plan and usage of the moment. */
  | { usage: CataloguePath };

export function apiBanner(
  props: { scene: UsageScene } & BannerLink,
): ReactNode {
  const { scene } = props;
  const plansAction = (label: string) =>
    "usage" in props ? (
      <LinkAction to={props.usage} tone="primary" size="sm">
        Plan and usage
      </LinkAction>
    ) : (
      <LinkAction to={props.plans} tone="primary" size="sm">
        {label}
      </LinkAction>
    );
  if (scene === "payment-failed" || scene === "payment-failed-over-limit") {
    return (
      <Notice
        tone="danger"
        title={`Stripe could not charge your card on ${paymentDates.failed}`}
        actions={
          <Action tone="primary" size="sm" icon={ExternalLink}>
            Update card
          </Action>
        }
      >
        Your plan stays active while Stripe tries again on {paymentDates.retry}.
        Jobs keep running.
      </Notice>
    );
  }
  if (scene === "moved-to-free") {
    return (
      <Notice
        tone="warning"
        title="Your paid plan ended after failed payments"
        actions={plansAction("Choose a plan")}
      >
        {workspace.name} is on Free now. Jobs keep running on all own machines.
      </Notice>
    );
  }
  if (!passedLimit.has(scene)) return undefined;
  const offer = apiOffer(scene);
  return (
    <Notice
      tone="warning"
      title={`${workspace.name} passed a limit of its plan`}
      actions={plansAction("Compare plans")}
    >
      Jobs keep running. {choiceName(offer)} covers your use for{" "}
      {choiceMonthly(offer)}: {choiceCapacity(offer)}.
    </Notice>
  );
}
