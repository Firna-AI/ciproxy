import type {
  CheckoutReturn,
  CheckoutReturnPage,
  CheckoutReviewPage,
  PaymentOutcome,
  PlanUsagePage,
} from "../src/api/billing-types.ts";
import {
  parseCheckoutReturnPage,
  parseCheckoutReview,
  parsePlanUsagePage,
} from "../src/api/billing-parse.ts";
import { readPaymentOutcome } from "../src/api/billing-readers.ts";
import {
  integer,
  object,
  optional,
  string,
  time,
} from "../src/api/parse-core.ts";
import type { RuntimeRedirects } from "./live-pages-config.ts";

type CheckoutBase = {
  version: 1;
  base_url: string;
  workspace_id: string;
  created_at: string;
  before: PlanUsagePage;
  review: CheckoutReviewPage;
  runtime_redirects: RuntimeRedirects;
};

export type CheckoutRecord = CheckoutBase &
  (
    | { phase: "reviewed" }
    | {
        phase: "submitted";
        payment: PaymentOutcome;
        checkout: CheckoutReturn | null;
        return_url: string | null;
      }
    | {
        phase: "returned";
        payment: PaymentOutcome;
        checkout: CheckoutReturn;
        return_url: string;
        returned: CheckoutReturnPage;
        after: PlanUsagePage;
      }
  );

function parsed<T>(read: (value: unknown) => T | undefined, value: unknown): T {
  const result = read(value);
  if (result === undefined)
    throw new Error("The private Checkout result is invalid.");
  return result;
}

function readSession(value: unknown): CheckoutReturn {
  return {
    session_id: string(object(value, "saved Checkout session").session_id),
  };
}

export function readCheckoutRecord(value: unknown): CheckoutRecord {
  const fields = object(value, "private Checkout result");
  if (integer(fields.version) !== 1)
    throw new Error("Unknown private Checkout result version.");
  const redirects = object(fields.runtime_redirects, "saved runtime redirects");
  const base: CheckoutBase = {
    version: 1,
    base_url: string(fields.base_url),
    workspace_id: string(fields.workspace_id),
    created_at: time(fields.created_at),
    before: parsed(parsePlanUsagePage, fields.before),
    review: parsed(parseCheckoutReview, fields.review),
    runtime_redirects: {
      success_url: string(redirects.success_url),
      cancel_url: string(redirects.cancel_url),
      portal_return_url: optional(redirects.portal_return_url, string),
    },
  };
  if (fields.phase === "reviewed") return { ...base, phase: "reviewed" };
  if (fields.phase === "submitted")
    return {
      ...base,
      phase: "submitted",
      payment: readPaymentOutcome(fields.payment),
      checkout: optional(fields.checkout, readSession),
      return_url: optional(fields.return_url, string),
    };
  if (fields.phase === "returned")
    return {
      ...base,
      phase: "returned",
      payment: readPaymentOutcome(fields.payment),
      checkout: readSession(fields.checkout),
      return_url: string(fields.return_url),
      returned: parsed(parseCheckoutReturnPage, fields.returned),
      after: parsed(parsePlanUsagePage, fields.after),
    };
  throw new Error("Unknown private Checkout result phase.");
}
