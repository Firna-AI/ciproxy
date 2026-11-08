// The reader of the one page banner. GET /banner returns it, and every page
// response that carries a `banner` field holds the same generated value, so a
// page reader keeps that field with `readBannerField`. The shell shows only
// the banner of GET /banner.

import type {
  BannerAction,
  InactiveOrganization,
  PageBanner,
  PageBannerView,
} from "./billing-types.ts";
import { readPlanOffer } from "./billing-readers.ts";
import {
  array,
  object,
  oneOf,
  optional,
  ParseError,
  parser,
  string,
  time,
} from "./parse-core.ts";
import { readActivation } from "./parse-links.ts";

const readAction = oneOf<BannerAction>({
  update_card: true,
  choose_plan: true,
  ask_admin_or_billing: true,
});

function readInactiveOrganization(value: unknown): InactiveOrganization {
  const fields = object(value, "inactive organization");
  return {
    id: string(fields.id),
    login: string(fields.login),
    activation: readActivation(fields.activation),
  };
}

export function readPageBanner(value: unknown): PageBanner {
  const fields = object(value, "banner");
  switch (fields.kind) {
    case "payment_failed":
      return {
        kind: "payment_failed",
        action: readAction(fields.action),
        failed_at: optional(fields.failed_at, time),
        next_retry_at: optional(fields.next_retry_at, time),
      };
    case "subscription_ended":
      return {
        kind: "subscription_ended",
        action: readAction(fields.action),
        until: time(fields.until),
      };
    case "inactive_organizations":
      return {
        kind: "inactive_organizations",
        organizations: array(fields.organizations, readInactiveOrganization),
      };
    case "soft_limit":
      return {
        kind: "soft_limit",
        action: readAction(fields.action),
        offer: readPlanOffer(fields.offer),
      };
    default:
      throw new ParseError("banner kind");
  }
}

/** The `banner` field of a page response: absent or null is no banner. */
export function readBannerField(value: unknown): PageBanner | null {
  return optional(value, readPageBanner);
}

export const parsePageBannerView = parser((value): PageBannerView => {
  const fields = object(value, "banner view");
  return { banner: readBannerField(fields.banner) };
});
