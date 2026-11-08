// The readers of the billing pages: Plan and usage, Plans, the checkout
// review, the checkout return, and the portal link. Each keeps the `banner`
// field of its response, which the shell does not render.

import type {
  CheckoutReturnPage,
  CheckoutReviewPage,
  PlanCard,
  PlanFeature,
  PlanUsagePage,
  PlansPage,
  PortalLink,
} from "./billing-types.ts";
import { readBannerField } from "./banner-reader.ts";
import {
  readAddOns,
  readPaymentOutcome,
  readPlanOffer,
  readPlanStatus,
} from "./billing-readers.ts";
import {
  array,
  boolean,
  date,
  decimal,
  fraction,
  integer,
  object,
  optional,
  parser,
  string,
  time,
} from "./parse-core.ts";

export function readPlanUsagePage(value: unknown): PlanUsagePage {
  const fields = object(value, "plan and usage");
  return {
    status: readPlanStatus(fields.status),
    current: readPlanOffer(fields.current),
    offer: readPlanOffer(fields.offer),
    starts: decimal(fields.starts),
    machines: decimal(fields.machines),
    organizations: decimal(fields.organizations),
    expected_starts: fraction(fields.expected_starts),
    starts_over: fraction(fields.starts_over),
    machines_over: decimal(fields.machines_over),
    organizations_over: decimal(fields.organizations_over),
    crossing_day: optional(fields.crossing_day, date),
    banner: readBannerField(fields.banner),
  };
}

function readFeature(value: unknown): PlanFeature {
  const fields = object(value, "plan feature");
  return {
    name: string(fields.name),
    coming_later: boolean(fields.coming_later),
  };
}

function readPlanCard(value: unknown): PlanCard {
  const fields = object(value, "plan card");
  return {
    plan: readPlanOffer(fields.plan),
    current: boolean(fields.current),
    covers_use: boolean(fields.covers_use),
    can_choose: boolean(fields.can_choose),
    history_days: integer(fields.history_days),
    organizations: optional(fields.organizations, integer),
    features: array(fields.features, readFeature),
    initial_add_ons: readAddOns(fields.initial_add_ons),
  };
}

export const parsePlanUsagePage = parser(readPlanUsagePage);

export const parsePlansPage = parser((value): PlansPage => {
  const fields = object(value, "plans");
  return {
    usage: readPlanUsagePage(fields.usage),
    plans: array(fields.plans, readPlanCard),
  };
});

export const parseCheckoutReview = parser((value): CheckoutReviewPage => {
  const fields = object(value, "checkout review");
  return {
    choice: readPlanOffer(fields.choice),
    today_cents: decimal(fields.today_cents),
    billing_day: integer(fields.billing_day),
    prorated: boolean(fields.prorated),
    period_end: time(fields.period_end),
    banner: readBannerField(fields.banner),
  };
});

export const parseCheckoutReturnPage = parser((value): CheckoutReturnPage => {
  const fields = object(value, "checkout return");
  return {
    outcome: readPaymentOutcome(fields.outcome),
    banner: readBannerField(fields.banner),
  };
});

export const parsePaymentOutcome = parser(readPaymentOutcome);

export const parsePortalLink = parser((value): PortalLink => {
  const fields = object(value, "portal link");
  return { url: string(fields.url) };
});
