import { resolve } from "node:path";

import { storageKeys } from "../src/state/storage.ts";
import { exerciseCase, copyFailure } from "./browser-actions.ts";
import { buildNavigation } from "./browser-build.ts";
import { smokeCases, type SmokeCase } from "./browser-cases.ts";
import type { BrowserContext, Page } from "./browser-driver.ts";
import { control, viewportControls, visible } from "./browser-fixture.ts";
import { shellJourney, staticReloads } from "./browser-journeys.ts";

export type SmokeRun = {
  name: string;
  init?: string;
  check(page: Page, context: BrowserContext, width: number): Promise<void>;
};

function screenSeed(item: SmokeCase): string | undefined {
  const storage =
    item.name === "auth-expired"
      ? { ...item.storage, [storageKeys.signInEnds]: "2026-11-06T11:00:00Z" }
      : item.storage;
  const entries =
    storage === undefined
      ? undefined
      : Object.entries(storage)
          .map(([key, value]) => {
            const area =
              key === storageKeys.installStarted
                ? "sessionStorage"
                : "localStorage";
            return `${area}.setItem(${JSON.stringify(key)}, ${JSON.stringify(value)})`;
          })
          .join(";\n");
  // Seed this context once. A document reload must keep product updates.
  return entries === undefined
    ? undefined
    : `if (sessionStorage.getItem("ciproxy.fixture-seeded") !== "1") {\n${entries};\nsessionStorage.setItem("ciproxy.fixture-seeded", "1");\n}`;
}

function screenRun(item: SmokeCase, origin: string, output: string): SmokeRun {
  return {
    name: item.name,
    init: screenSeed(item),
    async check(page, context, width) {
      await control(origin, { scenario: item.scenario });
      await page.goto(origin + item.path, { waitUntil: "domcontentloaded" });
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
      await exerciseCase(page, context, item, origin, width);
    },
  };
}

/** The ordered executable cases used for both selection and execution. */
export function createSmokeRegistry(
  origin: string,
  output: string,
): readonly SmokeRun[] {
  return [
    ...smokeCases.map((item) => screenRun(item, origin, output)),
    ...(["link", "programmatic", "history"] as const).map((mode): SmokeRun => ({
      name: `build-${mode}`,
      check: (page) => buildNavigation(page, origin, mode),
    })),
    ...(["add", "switch", "sign-out"] as const).map((journey): SmokeRun => ({
      name: `shell-${journey}`,
      check: (page) => shellJourney(page, origin, journey),
    })),
    {
      name: "copy-failure",
      // Arrange a denied clipboard. A user can deny this browser permission.
      init: 'Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: async () => { throw new Error("Denied") } } })',
      check: (page) => copyFailure(page, origin),
    },
    { name: "static-reloads", check: (page) => staticReloads(page, origin) },
  ];
}

/** Resolve every requested nonempty name before the run has side effects. */
export function selectSmokeRuns(
  registry: readonly SmokeRun[],
  args: readonly string[],
  warn: (message: string) => void,
): readonly SmokeRun[] {
  const flags = args.filter((value) => value.startsWith("--case="));
  const requested = new Set<string>();
  for (const flag of flags) {
    for (const segment of flag.slice(7).split(",")) {
      const name = segment.trim();
      if (name === "") warn("Ignored an empty smoke case name.");
      else requested.add(name);
    }
  }
  const known = new Set(registry.map((run) => run.name));
  const unknown = [...requested].filter((name) => !known.has(name));
  if (unknown.length > 0)
    throw new Error(`Unknown smoke cases: ${unknown.join(", ")}`);
  if (flags.length > 0 && requested.size === 0) {
    throw new Error("No smoke case names remain after empty segments.");
  }
  return requested.size === 0
    ? registry
    : registry.filter((run) => requested.has(run.name));
}
