// The banner of a passed limit, a failed payment, or an ended plan as the API
// words it, on every page: the known moments of usage-banner.tsx and the
// billing pages of the current API. The API gives the dates of a failed
// payment and the offer that covers the use, but not the plan, the limit that
// passed, or the day on which a paid plan ended. So the banner of a passed
// limit and of an ended plan stays general. The product counts every billing
// page as Plan and usage, so there the banner of a passed limit compares the
// plans, and on every other page it opens Plan and usage. On Plans itself, a
// link to Plans would only reload the page, so it opens Plan and usage. A
// person who cannot pay reads who can, in place of the action.

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
import { AskPayer } from "./role-actions.tsx";

/** Where the banner of a passed limit or of an ended plan leads. */
type BannerLink =
  /** The Plans screen of the moment, from each billing page but Plans. */
  | { plans: CataloguePath }
  /** On Plans itself: Plan and usage of the moment. */
  | { usage: CataloguePath }
  /**
   * On any other page: Plan and usage of the moment for a passed limit, and
   * its Plans for an ended plan.
   */
  | { other: { usage: CataloguePath; plans: CataloguePath } };

function PageLink({ to, children }: { to: CataloguePath; children: string }) {
  return (
    <LinkAction to={to} tone="primary" size="sm">
      {children}
    </LinkAction>
  );
}

export function apiBanner(
  props: {
    scene: UsageScene;
    /** An Admin or a Billing person pays; a Viewer reads who can. */
    canPay?: boolean;
  } & BannerLink,
): ReactNode {
  const { scene, canPay = true } = props;
  const act = (action: ReactNode) => (canPay ? action : <AskPayer />);
  const plansAction = (label: string) =>
    "other" in props ? (
      <PageLink to={props.other.plans}>{label}</PageLink>
    ) : "usage" in props ? (
      <PageLink to={props.usage}>Plan and usage</PageLink>
    ) : (
      <PageLink to={props.plans}>{label}</PageLink>
    );
  if (scene === "payment-failed" || scene === "payment-failed-over-limit") {
    return (
      <Notice
        tone="danger"
        title={`Stripe could not charge your card on ${paymentDates.failed}`}
        actions={act(
          <Action tone="primary" size="sm" icon={ExternalLink}>
            Update card
          </Action>,
        )}
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
        actions={act(plansAction("Choose a plan"))}
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
      actions={act(
        "other" in props ? (
          <PageLink to={props.other.usage}>Plan and usage</PageLink>
        ) : (
          plansAction("Compare plans")
        ),
      )}
    >
      Jobs keep running. {choiceName(offer)} covers your use for{" "}
      {choiceMonthly(offer)}: {choiceCapacity(offer)}.
    </Notice>
  );
}
