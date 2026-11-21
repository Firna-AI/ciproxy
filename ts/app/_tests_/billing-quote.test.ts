import assert from "node:assert/strict";
import test from "node:test";

import type {
  CheckoutReviewPage,
  PageBanner,
  PlanUsagePage,
} from "../src/api/billing-types.ts";
import type { AppServices } from "../src/services/services.ts";
import { createPagePayment } from "../src/state/billing-flows.ts";
import type { BannerSnapshot } from "../src/state/billing-banner.ts";
import type { PaymentSnapshot } from "../src/state/billing-payment.ts";
import { bannerStore, paymentGate, usageStore } from "../src/state/billing.ts";
import {
  checkoutSummary,
  type SummaryState,
} from "../src/view/billing-checkout-summary.ts";
import { outcomeModel } from "../src/view/billing-outcome.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import {
  billingApiPaths,
  offer,
  review,
  selection,
  signedInWith,
  teamOffer,
  unavailable,
  usagePage,
} from "./billing-support.ts";
import { deferred, response, settled } from "./support.ts";

const quote = {
  id: "023e4567-e89b-12d3-a456-426614174000",
  expires_at: "2026-11-06T12:30:00Z",
};
const business = offer(selection("business"), "29900", "150000", "10");
const reviewed = {
  ...review(business),
  prorated: true,
  quote,
} satisfies CheckoutReviewPage;
const refused = {
  code: "review_required",
  message: "Review the change again before payment.",
} as const;
const workspace = { id: workspaceId, name: "mokly-ai" };
const previousBanner: PageBanner = {
  kind: "soft_limit",
  action: "choose_plan",
  offer: teamOffer,
};

function currentTeam(): PlanUsagePage {
  const page = usagePage({ current: teamOffer, crossing_day: null });
  page.status.selection = selection("team");
  page.status.subscription = "active";
  return page;
}

function summaryOf(
  app: AppServices,
  payment: PaymentSnapshot,
  page: CheckoutReviewPage = reviewed,
): SummaryState {
  const usage = usageStore(app, workspaceId).state.get();
  return checkoutSummary({
    review: {
      wanted: page.choice.selection,
      review: { status: "ready", page },
    },
    payment,
    gate: paymentGate(app, workspaceId),
    usage: usage.status === "ready" ? usage.page : null,
    workspace: workspace.name,
  });
}

function shape(summary: SummaryState) {
  return summary.kind === "blocked"
    ? [summary.kind, summary.title, summary.action?.kind ?? null]
    : [summary.kind, null, null];
}

test("a refused paid quote stays submitting until Plan and usage refresh ends", async () => {
  const held = deferred<Response>();
  const { app, calls } = await signedInWith(
    "admin",
    response(currentTeam()),
    response(refused, 409),
    held.promise,
  );
  await usageStore(app, workspaceId).load();
  const payment = createPagePayment(app, workspaceId);
  const sending = payment.submit({ kind: "plan", review: reviewed });
  await settled();
  try {
    assert.equal(payment.state.get().status, "submitting");
    assert.equal(summaryOf(app, payment.state.get()).kind, "ready");
  } finally {
    held.resolve(response(currentTeam()));
    await sending;
  }
  assert.deepEqual(payment.state.get(), {
    status: "failed",
    submission: { kind: "plan", review: reviewed },
    failure: { kind: "problem", status: 409, problem: refused },
    reading: false,
  });
  assert.deepEqual(shape(summaryOf(app, payment.state.get())), [
    "blocked",
    "This review is out of date",
    "review-again",
  ]);
  assert.equal(
    outcomeModel(
      payment.state.get(),
      workspace,
      usageStore(app, workspaceId).state.get(),
    ),
    null,
  );
  assert.deepEqual(
    calls.map((call) => `${call.init.method} ${call.url}`),
    [
      `GET ${billingApiPaths.usage}`,
      `POST ${billingApiPaths.change}`,
      `GET ${billingApiPaths.usage}`,
    ],
  );
});

test("a refused quote offers review again only from fresh billing facts", async () => {
  const currentBusiness = usagePage({ current: business });
  currentBusiness.status.selection = selection("business");
  currentBusiness.status.subscription = "active";
  const pending = currentTeam();
  pending.status.payment_pending = true;
  const previous = currentTeam();
  previous.banner = previousBanner;
  const cases: {
    name: string;
    pages: Response[];
    expected: [string, string, string | null];
    banner: BannerSnapshot;
  }[] = [
    {
      name: "failed first read",
      pages: [response(unavailable, 503)],
      expected: ["blocked", "Unable to load plan and usage", "reload"],
      banner: { status: "loading" },
    },
    {
      name: "failed refresh with a prior page",
      pages: [response(previous), response(unavailable, 503)],
      expected: ["blocked", "Unable to load plan and usage", "reload"],
      banner: { status: "ready", banner: previousBanner, refreshFailure: null },
    },
    {
      name: "reviewed tier is current",
      pages: [response(currentBusiness)],
      expected: ["blocked", "Business is your current plan", null],
      banner: { status: "ready", banner: null, refreshFailure: null },
    },
    {
      name: "a payment is pending",
      pages: [response(pending)],
      expected: [
        "blocked",
        "A payment or a billing change is in progress",
        "usage",
      ],
      banner: { status: "ready", banner: null, refreshFailure: null },
    },
    {
      name: "fresh ordinary status",
      pages: [response(currentTeam())],
      expected: ["blocked", "This review is out of date", "review-again"],
      banner: { status: "ready", banner: null, refreshFailure: null },
    },
  ];
  const payment: PaymentSnapshot = {
    status: "failed",
    submission: { kind: "plan", review: reviewed },
    failure: { kind: "problem", status: 409, problem: refused },
    reading: false,
  };
  for (const entry of cases) {
    const { app, calls } = await signedInWith("admin", ...entry.pages);
    for (let read = 0; read < entry.pages.length; read += 1) {
      await usageStore(app, workspaceId).load();
    }
    assert.deepEqual(
      shape(summaryOf(app, payment)),
      entry.expected,
      entry.name,
    );
    assert.deepEqual(
      bannerStore(app, workspaceId).state.get(),
      entry.banner,
      entry.name,
    );
    assert.deepEqual(
      calls.map((call) => `${call.init.method} ${call.url}`),
      entry.pages.map(() => `GET ${billingApiPaths.usage}`),
      entry.name,
    );
  }
});

test("only a plan quote refusal suppresses the separate outcome notice", () => {
  const current = {
    status: "ready",
    page: currentTeam(),
    refreshFailure: null,
  } as const;
  const failure = { kind: "problem", status: 409, problem: refused } as const;
  const plan = outcomeModel(
    {
      status: "failed",
      submission: { kind: "plan", review: reviewed },
      failure,
      reading: false,
    },
    workspace,
    current,
  );
  const addOns = outcomeModel(
    {
      status: "failed",
      submission: { kind: "add-ons", quote: teamOffer },
      failure,
      reading: false,
    },
    workspace,
    current,
  );
  assert.equal(plan, null);
  assert.deepEqual(
    [addOns?.title, addOns?.actions.map((action) => action.kind)],
    ["The payment did not start", ["retry"]],
  );
});
