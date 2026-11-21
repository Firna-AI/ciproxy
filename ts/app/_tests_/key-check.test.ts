import assert from "node:assert/strict";
import test from "node:test";

import {
  chooseWallet,
  finishCheck,
  nextAttempt,
  uncheckedKey,
} from "../src/state/key-check.ts";
import { walletOptions } from "../src/view/boat.ts";
import { boatWallets } from "../scripts/fixtures-boat.ts";

test("the likely wallet of the key check starts chosen, not the first wallet", () => {
  const started = nextAttempt(uncheckedKey);
  const done = finishCheck(started, started.attempt, boatWallets());
  assert.equal(done.checked?.chosen, "boat-org-1001");
  assert.deepEqual(walletOptions(done.checked?.wallets ?? []), [
    { value: "personal", title: "Mira Patel", body: "Personal wallet" },
    {
      value: "boat-org-1001",
      title: "northstar",
      body: "Boat organization wallet",
    },
    {
      value: "boat-org-1002",
      title: "orbit",
      body: "Boat organization wallet",
    },
  ]);
});

test("a key edit or a newer check drops the wallets, and an older answer never applies", () => {
  const first = nextAttempt(uncheckedKey);
  const edited = nextAttempt(first);
  assert.deepEqual(finishCheck(edited, first.attempt, boatWallets()), {
    attempt: 2,
    checked: null,
  });
  const second = nextAttempt(edited);
  const done = finishCheck(second, second.attempt, boatWallets());
  assert.equal(done.checked?.chosen, "boat-org-1001");
  assert.deepEqual(nextAttempt(done), { attempt: 4, checked: null });
});

test("a choice takes only a wallet of the checked key, and no check holds no wallet", () => {
  assert.deepEqual(chooseWallet(uncheckedKey, "personal"), uncheckedKey);
  const started = nextAttempt(uncheckedKey);
  const done = finishCheck(started, started.attempt, boatWallets());
  assert.equal(chooseWallet(done, "personal").checked?.chosen, "personal");
  assert.deepEqual(chooseWallet(done, "boat-org-9999"), done);
  const unlisted = { ...boatWallets(), selected_wallet: "boat-org-9999" };
  assert.equal(
    finishCheck(started, started.attempt, unlisted).checked?.chosen,
    null,
  );
});
