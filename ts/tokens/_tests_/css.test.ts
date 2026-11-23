import { deepStrictEqual, strictEqual } from "node:assert";
import { test } from "node:test";

import { colorDeclarations, fontFaceCss } from "../css.ts";

test("CSS properties convert camel case names and preserve group prefixes", () => {
  deepStrictEqual(
    colorDeclarations("status", { primaryDeep: "#000", controlBorder: "#fff" }),
    ["--c-status-primary-deep:#000", "--c-status-control-border:#fff"],
  );
});

test("font faces declare the variable weight range", () => {
  strictEqual(
    fontFaceCss({ "Geist Variable": "/assets/geist.woff2" }),
    '@font-face{font-family:"Geist Variable";' +
      'src:url("/assets/geist.woff2") format("woff2");' +
      "font-weight:100 900;font-style:normal;font-display:auto}",
  );
});
