import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

import { launchBrowser, type Page } from "./browser-driver.ts";
import { selectSmokeRuns } from "./browser-registry.ts";
import { fixtureTime } from "./fixtures-core.ts";
import { capture, type M20Case } from "./m20-browser-fixture.ts";
import { machineCases } from "./m20-browser-machines.ts";
import { settingsCases } from "./m20-browser-settings.ts";

const origin = process.env.CIPROXY_HARNESS_URL ?? "http://127.0.0.1:4341";
const output = resolve(
  process.env.CIPROXY_SMOKE_OUTPUT ?? ".context/ciproxy-mvp-m20/browser-smoke",
);
const registry = [
  ...machineCases(origin, output),
  ...settingsCases(origin, output),
];
const selected = selectSmokeRuns(registry, process.argv.slice(2), (message) =>
  process.stderr.write(`WARNING ${message}\n`),
);
const results: {
  name: string;
  width: number;
  status: "pass" | "fail";
  duration_ms: number;
  reason?: string;
}[] = [];

await mkdir(output, { recursive: true });
const browser = await launchBrowser();

async function run(entry: M20Case, width: number): Promise<void> {
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
  let page: Page | undefined;
  let failed = false;
  const errors: string[] = [];
  try {
    page = await context.newPage();
    page.on("pageerror", () => {
      errors.push("pageerror");
    });
    page.on("console", (message) => {
      const expected =
        entry.expectedHttp?.some((status) =>
          message
            .text()
            .startsWith(
              `Failed to load resource: the server responded with a status of ${status}`,
            ),
        ) ?? false;
      if (message.type() === "error" && !expected) errors.push("console-error");
    });
    await page.clock.install({ time: new Date(fixtureTime) });
    await entry.check(page, context, width);
    assert.deepEqual(
      errors,
      [],
      "The page emitted a JavaScript or console error",
    );
    await capture(page, width, output, entry.name);
    results.push({
      name: entry.name,
      width,
      status: "pass",
      duration_ms: performance.now() - started,
    });
    process.stdout.write(`PASS ${entry.name} ${width}\n`);
  } catch (error) {
    failed = true;
    const reason = (
      error instanceof Error ? error.message : "Unknown browser failure"
    ).replace(/([?&][a-zA-Z_][a-zA-Z0-9_-]*=)[^\s&"')]+/g, "$1[redacted]");
    results.push({
      name: entry.name,
      width,
      status: "fail",
      duration_ms: performance.now() - started,
      reason,
    });
    await page
      ?.screenshot({
        path: resolve(output, `${entry.name}-${width}-failed.png`),
        fullPage: true,
      })
      .catch(() => undefined);
    process.stderr.write(`FAIL ${entry.name} ${width}: ${reason}\n`);
  } finally {
    await context.tracing.stop(
      failed
        ? { path: resolve(output, `${entry.name}-${width}-failed.zip`) }
        : {},
    );
    await context.close();
  }
}

try {
  const names = new Set(selected.map((entry) => entry.name));
  for (const width of [1440, 720, 390])
    for (const entry of registry.filter((item) => names.has(item.name)))
      await run(entry, width);
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
