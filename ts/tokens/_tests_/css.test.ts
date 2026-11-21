import { deepStrictEqual } from "node:assert";
import { test } from "node:test";

import { colorDeclarations } from "../css.ts";

test("CSS properties convert camel case names and preserve group prefixes", () => {
  deepStrictEqual(
    colorDeclarations("status", { primaryDeep: "#000", controlBorder: "#fff" }),
    ["--c-status-primary-deep:#000", "--c-status-control-border:#fff"],
  );
});
