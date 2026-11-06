// Requires a running harness-server.ts. Screenshots contain fixture data only.
import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

import { storageKeys } from "../src/state/storage.ts";
import { exerciseCase, copyFailure } from "./browser-actions.ts";
import { buildNavigation } from "./browser-build.ts";
import { smokeCases } from "./browser-cases.ts";
import {
  launchBrowser,
  type BrowserContext,
  type Page,
} from "./browser-driver.ts";
import { control, viewportControls, visible } from "./browser-fixture.ts";
import { shellJourney, staticReloads } from "./browser-journeys.ts";
import { fixtureTime } from "./fixtures-core.ts";

const origin = process.env.CIPROXY_HARNESS_URL ?? "http://127.0.0.1:4340";
const output = resolve(
  process.env.CIPROXY_SMOKE_OUTPUT ?? ".context/ciproxy-mvp-m19/browser-smoke",
);
const selected = process.argv
  .find((value) => value.startsWith("--case="))
  ?.slice(7);
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
              ].includes(name)
            ? 503
            : null;
      if (
        message.type() === "error" &&
        !(
          expectedStatus !== null &&
          message
            .text()
            .startsWith(
              `Failed to load resource: the server responded with a status of ${expectedStatus}`,
            )
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
    for (const item of smokeCases) {
      if (selected !== undefined && selected !== item.name) continue;
      const init =
        item.name === "auth-expired"
          ? `localStorage.setItem(${JSON.stringify(storageKeys.signInEnds)}, "2026-11-06T11:00:00Z")`
          : undefined;
      await run(
        item.name,
        width,
        async (page, context) => {
          await control(origin, { scenario: item.scenario });
          await page.goto(origin + item.path, {
            waitUntil: "domcontentloaded",
          });
          await visible(
            page.getByRole("heading", { name: item.heading, exact: true }),
          );
          if (item.text !== undefined)
            await visible(page.getByText(item.text, { exact: true }));
          await viewportControls(page, width);
          await page.screenshot({
            path: resolve(output, `${item.name}-${width}-initial.png`),
            fullPage: true,
          });
          await exerciseCase(page, context, item, origin);
        },
        init,
      );
    }
    for (const mode of ["link", "programmatic", "history"] as const) {
      if (selected !== undefined && selected !== `build-${mode}`) continue;
      await run(`build-${mode}`, width, (page) =>
        buildNavigation(page, origin, mode),
      );
    }
    for (const journey of ["add", "switch", "sign-out"] as const) {
      if (selected !== undefined && selected !== `shell-${journey}`) continue;
      await run(`shell-${journey}`, width, (page) =>
        shellJourney(page, origin, journey),
      );
    }
    if (selected === undefined || selected === "copy-failure") {
      // Arrange a denied clipboard. A user can deny this browser permission.
      await run(
        "copy-failure",
        width,
        (page) => copyFailure(page, origin),
        'Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: async () => { throw new Error("Denied") } } })',
      );
    }
    if (selected === undefined || selected === "static-reloads")
      await run("static-reloads", width, (page) => staticReloads(page, origin));
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
