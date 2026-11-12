import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

import { launchBrowser, type Browser, type Page } from "./browser-driver.ts";
import type { SmokeRun } from "./browser-registry.ts";
import {
  capturePage,
  configureSuccessEvidence,
} from "./browser-pages-helpers.ts";
import { fixtureTime } from "./fixtures-core.ts";

export type PagesSmokeRun = SmokeRun & {
  expectedHttpStatuses?: readonly number[];
  cleanup?: () => Promise<void>;
};
export type PagesRunResult = {
  name: string;
  width: number;
  attempt: number;
  status: "pass" | "fail";
  duration_ms: number;
  screenshot?: string;
  trace?: string;
  reason?: string;
};

function safeReason(error: unknown): string {
  return (
    error instanceof Error ? error.message : "Unknown page smoke failure"
  ).replace(/([?&][a-zA-Z_][a-zA-Z0-9_-]*=)[^\s&"')]+/g, "$1[redacted]");
}

async function runCase(
  browser: Browser,
  entry: PagesSmokeRun,
  width: number,
  attempt: number,
  output: string,
): Promise<PagesRunResult> {
  const started = performance.now();
  const context = await browser.newContext({
    viewport: { width, height: 1000 },
    locale: "en-GB",
    timezoneId: "UTC",
  });
  context.setDefaultTimeout(10_000);
  let page: Page | undefined;
  const evidence = attempt === 1;
  const errors: string[] = [];
  const result: PagesRunResult = {
    name: entry.name,
    width,
    attempt,
    status: "fail",
    duration_ms: 0,
  };
  try {
    if (evidence)
      await context.tracing.start({
        screenshots: true,
        snapshots: true,
        sources: false,
      });
    if (entry.init !== undefined) await context.addInitScript(entry.init);
    page = await context.newPage();
    configureSuccessEvidence(page, evidence);
    page.on("pageerror", () => {
      errors.push("pageerror");
    });
    page.on("console", (message) => {
      const expected = entry.expectedHttpStatuses?.some((status) =>
        message
          .text()
          .startsWith(
            `Failed to load resource: the server responded with a status of ${status}`,
          ),
      );
      if (message.type() === "error" && !expected) errors.push("console-error");
    });
    await page.clock.install({ time: new Date(fixtureTime) });
    await entry.check(page, context, width);
    assert.deepEqual(
      errors,
      [],
      "The page emitted a JavaScript or console error.",
    );
    result.screenshot = await capturePage(page, output, entry.name, width);
    result.status = "pass";
  } catch (error) {
    result.reason = safeReason(error);
    if (evidence)
      result.trace = resolve(output, `${entry.name}-${width}-failed.zip`);
    if (page !== undefined) {
      const screenshot = resolve(output, `${entry.name}-${width}-failed.png`);
      try {
        await page.screenshot({ path: screenshot, fullPage: true });
        result.screenshot = screenshot;
      } catch {
        /* The trace still records a closed-page failure. */
      }
    }
  } finally {
    try {
      await entry.cleanup?.();
      if (evidence)
        await context.tracing.stop(
          result.trace === undefined ? {} : { path: result.trace },
        );
    } finally {
      await context.close();
      result.duration_ms = performance.now() - started;
    }
  }
  process.stdout.write(
    `${result.status === "pass" ? "PASS" : "FAIL"} ${entry.name} ${width} run ${attempt}\n`,
  );
  return result;
}

export async function runPagesSmoke(
  factory: (output: string) => readonly PagesSmokeRun[],
  selectedNames: ReadonlySet<string>,
  output: string,
  repeat: number,
  options: { widths?: readonly number[]; stopOnFailure?: boolean } = {},
): Promise<PagesRunResult[]> {
  const results: PagesRunResult[] = [];
  await mkdir(output, { recursive: true });
  const browser = await launchBrowser();
  try {
    runs: for (let attempt = 1; attempt <= repeat; attempt += 1) {
      const directory = resolve(
        output,
        `run-${String(attempt).padStart(2, "0")}`,
      );
      await mkdir(directory, { recursive: true });
      const registry = factory(directory).filter((entry) =>
        selectedNames.has(entry.name),
      );
      for (const width of options.widths ?? [1440, 720, 390]) {
        for (const entry of registry) {
          results.push(
            await runCase(browser, entry, width, attempt, directory),
          );
          await writeFile(
            resolve(output, "results.json"),
            JSON.stringify(results, null, 2) + "\n",
          );
          if (options.stopOnFailure && results.at(-1)?.status === "fail")
            break runs;
        }
      }
    }
  } finally {
    await browser.close();
    await writeFile(
      resolve(output, "results.json"),
      JSON.stringify(results, null, 2) + "\n",
    );
  }
  return results;
}
