import assert from "node:assert/strict";
import test from "node:test";

import {
  formatMeasuredGigabytes,
  formatMeasuredVcpu,
} from "../src/view/format.ts";

test("a measured figure shows one decimal, half up, grouped, and on one line", () => {
  assert.deepEqual(
    [0, 49, 50, 1549, 1550, 2000, 12345].map(formatMeasuredVcpu),
    [
      "0.0\u00a0vCPU",
      "0.0\u00a0vCPU",
      "0.1\u00a0vCPU",
      "1.5\u00a0vCPU",
      "1.6\u00a0vCPU",
      "2.0\u00a0vCPU",
      "12.3\u00a0vCPU",
    ],
  );
  assert.deepEqual(
    [
      "0",
      "49999999",
      "50000000",
      "5452595200",
      "6082000000",
      "7949999999",
      "7950000000",
      "1234567890123456789",
    ].map(formatMeasuredGigabytes),
    [
      "0.0\u00a0GB",
      "0.0\u00a0GB",
      "0.1\u00a0GB",
      "5.5\u00a0GB",
      "6.1\u00a0GB",
      "7.9\u00a0GB",
      "8.0\u00a0GB",
      "1,234,567,890.1\u00a0GB",
    ],
  );
});
