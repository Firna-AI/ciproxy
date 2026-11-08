import {
  billingHref,
  checkoutHref,
  checkoutReturnHref,
  plansHref,
} from "../src/view/billing-routes.ts";
import { askPayer } from "../src/view/billing-format.ts";
import { baseSelection } from "./fixtures-billing.ts";
import { fixtureCheckoutId } from "./fixtures-billing-payment.ts";
import { workspaceId } from "./fixtures-core.ts";
import { screenRuns, type PageScreen } from "./browser-pages-screens.ts";

export function billingScreenRuns(origin: string) {
  const path = billingHref(workspaceId);
  const plans = plansHref(workspaceId);
  const freeReview = checkoutHref(workspaceId, baseSelection("team"));
  const paidReview = checkoutHref(workspaceId, baseSelection("business"));
  const returned = checkoutReturnHref(workspaceId, fixtureCheckoutId);
  const usageCases = [
    ["usage-free", "m21-usage-free"],
    ["usage-paid", "m21-usage-paid"],
    ["usage-business", "m21-usage-business"],
    ["usage-soft-starts", "m21-usage-soft-starts"],
    ["usage-soft-machines", "m21-usage-soft-machines"],
    ["usage-soft-organizations", "m21-usage-soft-organizations"],
    ["usage-near-limit", "m21-usage-near-limit"],
    ["usage-payment-failed", "m21-usage-payment-failed"],
    ["usage-ended", "m21-usage-ended"],
    ["usage-pending", "m21-usage-pending"],
    ["usage-scheduled-free", "m21-usage-scheduled-free"],
    ["usage-operation-pending", "m21-usage-operation-pending"],
  ] as const;
  const screens: PageScreen[] = [
    ...usageCases.map(([name, scenario]): PageScreen => ({
      name,
      scenario,
      path,
      heading: "This period",
    })),
    {
      name: "usage-error",
      scenario: "m21-usage-error",
      path,
      heading: "Unable to load plan and usage",
      statuses: [503],
    },
    {
      name: "usage-viewer",
      scenario: "m21-viewer",
      path,
      heading: "This period",
      text: askPayer,
    },
    {
      name: "usage-billing",
      scenario: "m21-billing",
      path,
      heading: "This period",
      role: { role: "button", name: "Invoices and card" },
    },
    {
      name: "plans-free",
      scenario: "m21-plans-free",
      path: plans,
      heading: "Business",
    },
    {
      name: "plans-paid",
      scenario: "m21-plans-paid",
      path: plans,
      role: { role: "button", name: "Move to Free" },
    },
    {
      name: "plans-viewer",
      scenario: "m21-plans-viewer",
      path: plans,
      heading: "Business",
    },
    {
      name: "plans-pending",
      scenario: "m21-plans-pending",
      path: plans,
      text: "Stripe is confirming a payment",
    },
    {
      name: "plans-error",
      scenario: "m21-plans-error",
      path: plans,
      heading: "Unable to load the plans",
      statuses: [503],
    },
    {
      name: "review-free",
      scenario: "m21-review-free",
      path: freeReview,
      role: { role: "button", name: "Continue to payment" },
    },
    {
      name: "review-paid",
      scenario: "m21-review-paid",
      path: paidReview,
      role: { role: "button", name: "Move to Business" },
    },
    {
      name: "review-error",
      scenario: "m21-review-error",
      path: freeReview,
      text: "Unable to price this change",
      statuses: [503],
    },
    {
      name: "review-current-plan",
      scenario: "m21-review-same-tier-conflict",
      path: freeReview,
      text: "Team is your current plan",
      statuses: [409],
    },
    {
      name: "review-viewer",
      scenario: "m21-viewer",
      path: paidReview,
      heading: askPayer,
    },
    {
      name: "return-open",
      scenario: "m21-payment-open",
      pages: { returned: "open" },
      path: returned,
      heading: "Your payment is not finished",
    },
    {
      name: "return-confirming",
      scenario: "m21-payment-confirming",
      path: returned,
      heading: "Confirming your payment",
      response: { path: "checkout/return", method: "POST" },
    },
    {
      name: "return-confirmed",
      scenario: "m21-payment-confirmed",
      path: returned,
      heading: "Northstar is on Team",
    },
    {
      name: "return-unfinished",
      scenario: "m21-payment-unfinished",
      path: returned,
      heading: "Your plan did not change",
    },
    {
      name: "return-payment-required",
      scenario: "m21-payment-required",
      path: returned,
      heading: "Your bank needs to confirm this payment",
    },
    {
      name: "return-error",
      scenario: "m21-payment-return-error",
      path: returned,
      heading: "Unable to check the payment",
      statuses: [503],
    },
    {
      name: "return-viewer",
      scenario: "m21-viewer",
      path: returned,
      heading: askPayer,
    },
    {
      name: "return-missing-session",
      scenario: "m21-payment-open",
      path: `/checkout-return?workspace=${workspaceId}`,
      heading: "This page names no payment",
    },
    {
      name: "return-no-provenance",
      scenario: "m21-payment-open",
      path: `/checkout-return?session_id=${fixtureCheckoutId}`,
      heading: "This tab cannot tell the workspace of this payment",
    },
    {
      name: "banner-legacy-soft-limit",
      scenario: "soft-limit",
      path,
      text: "Northstar passed a limit of its plan",
    },
    {
      name: "banner-error",
      scenario: "m21-banner-error",
      path,
      text: "Unable to check the plan status",
      statuses: [503],
    },
  ];
  return screenRuns(origin, screens);
}
