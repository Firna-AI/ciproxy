import assert from "node:assert/strict";
import test from "node:test";

import type { PaymentOutcome } from "../src/api/billing-types.ts";
import { usageStore } from "../src/state/billing.ts";
import {
  createPagePayment,
  stripeReturnTarget,
} from "../src/state/billing-flows.ts";
import { checkoutQuery, stripeReturnHref } from "../src/view/billing-routes.ts";
import { otherWorkspaceId, workspaceId } from "../scripts/fixtures-core.ts";
import { review, usagePage } from "./billing-support.ts";
import { appWith, response } from "./support.ts";

const stripeUrl = "https://checkout.stripe.test/c/pay/cs_test_a";

test("normal workspace navigation cannot retarget a payment return", async () => {
  const { app, documents } = appWith(
    response(usagePage()),
    response({ state: "open", url: stripeUrl } satisfies PaymentOutcome),
  );
  app.rememberWorkspace(workspaceId);
  await usageStore(app, workspaceId).load();
  await createPagePayment(app, workspaceId).submit({
    kind: "plan",
    review: review(),
  });
  assert.deepEqual(documents, [stripeUrl]);
  // Another page of this browser opens a second workspace.
  app.rememberWorkspace(otherWorkspaceId);
  assert.equal(
    stripeReturnTarget(app, "checkout", "/checkout-return", {
      session_id: "cs_test_a",
    }),
    `/checkout-return?session_id=cs_test_a&workspace=${workspaceId}`,
  );
});

test("a return with no saved payment never takes the remembered workspace", () => {
  const { app } = appWith();
  app.rememberWorkspace(otherWorkspaceId);
  assert.equal(
    stripeReturnTarget(app, "checkout", "/checkout-return", {
      session_id: "cs_test_a",
    }),
    null,
  );
  assert.equal(stripeReturnTarget(app, "portal", "/billing", {}), null);
});

test("a workspace in the query wins over the saved one", () => {
  assert.equal(
    stripeReturnHref(
      "/checkout-return",
      { workspace: otherWorkspaceId, session_id: "cs_test_a" },
      workspaceId,
    ),
    null,
  );
});

test("a checkout query takes whole quantities and leaves the range to the API", () => {
  assert.deepEqual(
    checkoutQuery({
      tier: "team",
      start_blocks: "4294967295",
      machines: "007",
    }),
    { tier: "team", add_ons: { start_blocks: 4294967295, machines: 7 } },
  );
  for (const blocks of ["-1", "1.5", "", "9007199254740993", ["1", "2"]]) {
    assert.equal(
      checkoutQuery({ tier: "team", start_blocks: blocks, machines: "0" }),
      null,
      `start_blocks ${JSON.stringify(blocks)}`,
    );
  }
  assert.equal(
    checkoutQuery({ tier: "enterprise", start_blocks: "0", machines: "0" }),
    null,
  );
  assert.equal(checkoutQuery({ tier: "team", machines: "0" }), null);
});
