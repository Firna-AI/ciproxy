import assert from "node:assert/strict";
import test from "node:test";

import {
  centsToInput,
  formatAgo,
  formatCents,
  formatSince,
  parseDollars,
  waitText,
} from "../src/view/machines-format.ts";

const nbsp = "\u00a0";

test("money shows exact cents only when the amount has cents", () => {
  const cases: [string, string][] = [
    ["5340", "$53.40"],
    ["4900", "$49"],
    ["5", "$0.05"],
    ["0", "$0"],
    ["123456789", "$1,234,567.89"],
    ["900719925474099312", "$9,007,199,254,740,993.12"],
  ];
  for (const [cents, expected] of cases) {
    assert.equal(formatCents(cents), expected, cents);
  }
});

test("a typed monthly cost becomes integer cents, or null when it is no amount", () => {
  const cases: [string, string | null][] = [
    ["53.40", "5340"],
    ["53.4", "5340"],
    [" $1,234.5 ", "123450"],
    ["0", "0"],
    ["0.00", "0"],
    ["007", "700"],
    ["53.405", null],
    ["-1", null],
    ["1,23", null],
    ["", null],
    ["abc", null],
  ];
  for (const [typed, expected] of cases) {
    assert.equal(parseDollars(typed), expected, JSON.stringify(typed));
  }
  assert.equal(centsToInput("5340"), "53.40");
  assert.equal(centsToInput("4900"), "49");
  assert.equal(centsToInput("0"), "0");
});

test("the time since a contact is short in a table and spelled out in a sentence", () => {
  const now = Date.parse("2026-11-18T14:32:00Z");
  assert.equal(formatAgo("2026-11-18T14:31:57Z", now), "3s ago");
  assert.equal(formatAgo("2026-11-18T14:18:00Z", now), "14m ago");
  assert.equal(formatAgo("2026-11-18T11:32:00Z", now), "3h ago");
  assert.equal(formatSince("2026-11-18T14:18:00Z", now), `14${nbsp}minutes`);
  assert.equal(formatSince("2026-11-18T14:31:00Z", now), `1${nbsp}minute`);
  assert.equal(formatSince("2026-11-16T14:32:00Z", now), `2${nbsp}days`);
});

test("the overflow wait uses the names of Settings", () => {
  assert.equal(waitText(0), "At once");
  assert.equal(waitText(15), `15${nbsp}seconds`);
  assert.equal(waitText(60), `1${nbsp}minute`);
  assert.equal(waitText(120), `2${nbsp}minutes`);
});
