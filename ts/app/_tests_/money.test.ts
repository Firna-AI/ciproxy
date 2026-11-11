import assert from "node:assert/strict";
import test from "node:test";

import {
  dollarsText,
  formatCents,
  parseDollars,
  shareOf,
} from "../src/view/money.ts";

test("a typed limit becomes exact integer cents, and only above zero", () => {
  const accepted: [string, string][] = [
    ["200", "20000"],
    ["$200", "20000"],
    ["1,000", "100000"],
    ["49.5", "4950"],
    ["0.29", "29"],
    ["0.07", "7"],
    [".50", "50"],
    ["$.5", "50"],
    ["200.", "20000"],
    [" 12.34 ", "1234"],
    ["92233720368547758.07", "9223372036854775807"],
  ];
  for (const [typed, cents] of accepted) {
    assert.deepEqual(parseDollars(typed), { ok: true, cents }, typed);
  }
  for (const typed of ["", "0", "0.00", ".", "-5", "1.005", "10,00", "abc"]) {
    assert.deepEqual(parseDollars(typed), { ok: false }, typed);
  }
});

test("cents show as dollars with cents only when they are not whole", () => {
  assert.equal(formatCents("20000"), "$200");
  assert.equal(formatCents("6120"), "$61.20");
  assert.equal(formatCents("7"), "$0.07");
  assert.equal(formatCents("123456789"), "$1,234,567.89");
  assert.equal(dollarsText("20050"), "200.50");
  assert.equal(dollarsText("100000"), "1000");
});

test("a share of the limit stays between 0 and 1", () => {
  assert.equal(shareOf("4810", "20000"), 0.2405);
  assert.equal(shareOf("25000", "20000"), 1);
  assert.equal(shareOf("0", "20000"), 0);
});
