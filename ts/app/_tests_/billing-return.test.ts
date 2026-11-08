import assert from "node:assert/strict";
import test from "node:test";

import type { ApiResult } from "../src/api/client.ts";
import type {
  CheckoutReturnPage,
  PaymentOutcome,
  PlanUsagePage,
} from "../src/api/billing-types.ts";
import type { WorkspaceRole } from "../src/api/schema.ts";
import { workspaceHref } from "../src/routing/pages.ts";
import {
  confirmingAnswers,
  createReturnStore,
  returnPollInterval,
  type ReturnSnapshot,
} from "../src/state/billing-return.ts";
import {
  returnView,
  type ReturnAction,
} from "../src/view/billing-return-page.ts";
import { billingHref } from "../src/view/billing-routes.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import {
  conflict,
  selection,
  teamOffer,
  unavailable,
  usagePage,
} from "./billing-support.ts";
import type { Visibility } from "../src/state/poller.ts";
import { deferred, manualSchedule, settled } from "./support.ts";

type Reply =
  ApiResult<CheckoutReturnPage> | Promise<ApiResult<CheckoutReturnPage>>;

function answer(outcome: PaymentOutcome): ApiResult<CheckoutReturnPage> {
  return { ok: true, value: { outcome, banner: null } };
}

function refused(
  status: number,
  problem: typeof conflict | typeof unavailable,
) {
  return {
    ok: false,
    failure: { kind: "problem", status, problem },
  } satisfies ApiResult<CheckoutReturnPage>;
}

/** The tab watchers of `visibility` that nothing removed yet. */
function countWatchers(visibility: Visibility) {
  let active = 0;
  const counted: Visibility = {
    hidden: () => visibility.hidden(),
    watch(listener) {
      active += 1;
      const remove = visibility.watch(listener);
      let removed = false;
      return () => {
        if (!removed) active -= 1;
        removed = true;
        remove();
      };
    },
  };
  return { visibility: counted, active: () => active };
}

function returnWith(...replies: Reply[]) {
  const schedule = manualSchedule();
  const watchers = countWatchers(schedule.visibility);
  let checks = 0;
  let reads = 0;
  const store = createReturnStore({
    reconcile: async () => {
      checks += 1;
      const reply = replies.shift();
      assert.notEqual(reply, undefined, "Unexpected return check");
      if (reply === undefined) throw new Error("Missing reply");
      return reply;
    },
    mutate: (run) => run(),
    timers: schedule.timers,
    visibility: watchers.visibility,
    settled: async () => {
      reads += 1;
    },
  });
  return {
    store,
    schedule,
    checks: () => checks,
    reads: () => reads,
    watchers: watchers.active,
  };
}

test("a return page that closes stops checking, also with a check on its way", async () => {
  const held = deferred<ApiResult<CheckoutReturnPage>>();
  const { store, schedule, checks } = returnWith(held.promise);
  store.start();
  store.stop();
  held.resolve(answer({ state: "confirming" }));
  await settled();
  assert.equal(schedule.pending(), 0);
  schedule.hide(false);
  await settled();
  assert.equal(checks(), 1);
});

test("the return waits for the server, says that Stripe still confirms, then settles once", async () => {
  const { store, schedule, checks, reads } = returnWith(
    answer({ state: "confirming" }),
    refused(409, conflict),
    answer({ state: "confirming" }),
    answer({ state: "confirmed" }),
  );
  store.start();
  await settled();
  assert.deepEqual(store.state.get(), {
    status: "confirming",
    attempts: 1,
    still: false,
  });
  assert.deepEqual(schedule.delays, [returnPollInterval]);
  schedule.fire();
  await settled();
  schedule.fire();
  await settled();
  assert.deepEqual(store.state.get(), {
    status: "confirming",
    attempts: confirmingAnswers,
    still: true,
  });
  assert.equal(reads(), 0);
  schedule.fire();
  await settled();
  assert.deepEqual(store.state.get(), {
    status: "settled",
    outcome: { state: "confirmed" },
  });
  assert.equal(reads(), 1);
  assert.equal(schedule.pending(), 0);
  assert.equal(checks(), 4);
});

test("a failed check stops, shows the failure, and Try again checks the same session", async () => {
  const { store, schedule, checks } = returnWith(
    refused(503, unavailable),
    answer({ state: "unfinished" }),
  );
  store.start();
  await settled();
  const failed = store.state.get();
  assert.equal(failed.status, "failed");
  assert.equal(schedule.pending(), 0);
  store.retry();
  await settled();
  assert.deepEqual(store.state.get(), {
    status: "settled",
    outcome: { state: "unfinished" },
  });
  assert.equal(checks(), 2);
});

test("Try again after a failed check keeps one tab watcher, and leaving removes it", async () => {
  const held = deferred<ApiResult<CheckoutReturnPage>>();
  const { store, schedule, watchers } = returnWith(
    answer({ state: "confirming" }),
    refused(503, unavailable),
    held.promise,
  );
  store.start();
  await settled();
  assert.equal(watchers(), 1);
  // The check on the interval fails, and the cycle ends by itself.
  schedule.fire();
  await settled();
  assert.equal(store.state.get().status, "failed");
  store.retry();
  held.resolve(answer({ state: "confirming" }));
  await settled();
  assert.equal(watchers(), 1);
  store.stop();
  assert.equal(watchers(), 0);
});

test("after a confirmed payment, only an Admin gets Add machines", () => {
  const paidTeam = usagePage({ current: teamOffer, crossing_day: null });
  paidTeam.status.selection = selection("team");
  const workspace = { id: workspaceId, name: "mokly-ai" };
  const confirmed: ReturnSnapshot = {
    status: "settled",
    outcome: { state: "confirmed" },
  };
  const openUsage: ReturnAction = {
    kind: "link",
    label: "Open plan and usage",
    href: billingHref(workspaceId),
    primary: false,
  };
  const addMachines: ReturnAction = {
    kind: "link",
    label: "Add machines",
    href: workspaceHref("machines", workspaceId),
    primary: true,
  };
  const cases: readonly {
    name: string;
    role: WorkspaceRole;
    usage: PlanUsagePage | null;
    actions: readonly ReturnAction[];
  }[] = [
    {
      name: "Billing, paid Team",
      role: "billing",
      usage: paidTeam,
      actions: [openUsage],
    },
    {
      name: "Billing, no usage read",
      role: "billing",
      usage: null,
      actions: [openUsage],
    },
    {
      name: "Admin, paid Team",
      role: "admin",
      usage: paidTeam,
      actions: [addMachines, openUsage],
    },
  ];
  // One comparison of the whole table shows every row that differs.
  assert.deepEqual(
    cases.map(({ name, role, usage }) => {
      const view = returnView(confirmed, usage, workspace, role);
      return { name, actions: view.kind === "result" ? view.actions : null };
    }),
    cases.map(({ name, actions }) => ({ name, actions })),
  );
});
