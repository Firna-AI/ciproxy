// Requires a running harness-server.ts. Screenshots contain fixture data only.
import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

import {
  launchBrowser,
  type BrowserContext,
  type Page,
} from "./browser-driver.ts";
import { boatExpectedStatus } from "./browser-boat.ts";
import { viewportControls } from "./browser-fixture.ts";
import { createSmokeRegistry, selectSmokeRuns } from "./browser-registry.ts";
import { fixtureTime } from "./fixtures-core.ts";

const origin = process.env.CIPROXY_HARNESS_URL ?? "http://127.0.0.1:4340";
const output = resolve(
  process.env.CIPROXY_SMOKE_OUTPUT ?? ".context/ciproxy-mvp-m19/browser-smoke",
);
const selected = (() => {
  try {
    return selectSmokeRuns(
      createSmokeRegistry(origin, output),
      process.argv.slice(2),
      (message) => process.stderr.write(`WARNING ${message}\n`),
    );
  } catch (error) {
    process.stderr.write(
      `FAIL ${error instanceof Error ? error.message : "Unable to select smoke cases"}\n`,
    );
    process.exit(1);
  }
})();
const results: {
  name: string;
  width: number;
  status: "pass" | "fail";
  duration_ms: number;
  reason?: string;
}[] = [];
await mkdir(output, { recursive: true });
const browser = await launchBrowser();

async function run(
  name: string,
  width: number,
  check: (page: Page, context: BrowserContext) => Promise<void>,
  init?: string,
): Promise<void> {
  const started = performance.now();
  const context = await browser.newContext({
    viewport: { width, height: 1000 },
    locale: "en-GB",
    timezoneId: "UTC",
  });
  await context.tracing.start({
    screenshots: true,
    snapshots: true,
    sources: false,
  });
  let page: Page | null = null;
  let failed = false;
  const errors: string[] = [];
  try {
    if (init !== undefined) await context.addInitScript(init);
    page = await context.newPage();
    page.on("pageerror", () => {
      errors.push("pageerror");
    });
    page.on("console", (message) => {
      // Intentional HTTP failures are checked as rendered recovery states.
      const expectedStatus =
        name === "auth-expired"
          ? 401
          : [
                "setup-load-failure",
                "command-load-failure",
                "command-revoke-failure",
                "session-refresh-no-role",
                "session-refresh-workspace",
                "claim-failure-no-role",
                "claim-failure-workspace",
                "boat-setup-load-failure",
                "overflow-load-failure",
              ].includes(name)
            ? 503
            : name === "claim-failure-csrf"
              ? 403
              : (boatExpectedStatus.get(name) ?? null);
      if (
        message.type() === "error" &&
        !(
          (name === "claim-network-failure" &&
            message
              .text()
              .startsWith(
                "Failed to load resource: net::ERR_EMPTY_RESPONSE",
              )) ||
          (expectedStatus !== null &&
            message
              .text()
              .startsWith(
                `Failed to load resource: the server responded with a status of ${expectedStatus}`,
              ))
        )
      )
        errors.push("console-error");
    });
    await page.clock.install({ time: new Date(fixtureTime) });
    await check(page, context);
    assert.deepEqual(
      errors,
      [],
      "The page emitted a JavaScript or console error",
    );
    await viewportControls(page, width);
    await page.screenshot({
      path: resolve(output, `${name}-${width}.png`),
      fullPage: true,
    });
    results.push({
      name,
      width,
      status: "pass",
      duration_ms: performance.now() - started,
    });
    process.stdout.write(`PASS ${name} ${width}\n`);
  } catch (error) {
    failed = true;
    const reason = (
      error instanceof Error ? error.message : "Unknown browser failure"
    ).replace(/([?&][a-zA-Z_][a-zA-Z0-9_-]*=)[^\s&"')]+/g, "$1[redacted]");
    results.push({
      name,
      width,
      status: "fail",
      duration_ms: performance.now() - started,
      reason,
    });
    await page
      ?.screenshot({
        path: resolve(output, `${name}-${width}-failed.png`),
        fullPage: true,
      })
      .catch(() => undefined);
    process.stderr.write(`FAIL ${name} ${width}: ${reason}\n`);
  } finally {
    await context.tracing.stop(
      failed ? { path: resolve(output, `${name}-${width}-failed.zip`) } : {},
    );
    await context.close();
  }
}

try {
  for (const width of [1440, 720, 390]) {
    for (const entry of selected) {
      await run(
        entry.name,
        width,
        (page, context) => entry.check(page, context, width),
        entry.init,
      );
    }
  }
} finally {
  await browser.close();
  await writeFile(
    resolve(output, "results.json"),
    JSON.stringify(results, null, 2) + "\n",
  );
}

const failed = results.filter((result) => result.status === "fail").length;
process.stdout.write(
  `Checks ${results.length}; pass ${results.length - failed}; fail ${failed}; skip 0\n`,
);
if (results.length === 0 || failed > 0) process.exitCode = 1;
