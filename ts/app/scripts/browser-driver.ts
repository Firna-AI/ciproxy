// A small tool interface permits an operator-supplied playwright-core path.
// It adds no package dependency to the dashboard.
import { pathToFileURL } from "node:url";

import { fields } from "./harness-state.ts";

export type Locator = {
  all(): Promise<Locator[]>;
  waitFor(options?: {
    state?: "visible" | "hidden" | "attached";
    timeout?: number;
  }): Promise<void>;
  click(): Promise<void>;
  fill(value: string): Promise<void>;
  inputValue(): Promise<string>;
  focus(): Promise<void>;
  press(key: string): Promise<void>;
  getAttribute(name: string): Promise<string | null>;
  textContent(): Promise<string | null>;
  count(): Promise<number>;
  isEnabled(): Promise<boolean>;
  isChecked(): Promise<boolean>;
  boundingBox(): Promise<{
    x: number;
    y: number;
    width: number;
    height: number;
  } | null>;
};

export type Page = {
  goto(
    url: string,
    options?: { waitUntil: "domcontentloaded" },
  ): Promise<unknown>;
  reload(options?: { waitUntil: "domcontentloaded" }): Promise<unknown>;
  setViewportSize(size: { width: number; height: number }): Promise<void>;
  goBack(options?: { waitUntil: "domcontentloaded" }): Promise<unknown>;
  waitForURL(url: string | RegExp): Promise<void>;
  waitForResponse(url: RegExp): Promise<{
    headers(): Record<string, string>;
    finished(): Promise<unknown>;
  }>;
  url(): string;
  getByRole(
    role: string,
    options?: { name?: string | RegExp; exact?: boolean; checked?: boolean },
  ): Locator;
  getByText(text: string | RegExp, options?: { exact?: boolean }): Locator;
  getByLabel(text: string | RegExp, options?: { exact?: boolean }): Locator;
  waitForFunction<A>(check: (arg: A) => boolean, arg: A): Promise<unknown>;
  getByTestId(id: string): Locator;
  keyboard: { press(key: string): Promise<void> };
  screenshot(options: { path: string; fullPage: boolean }): Promise<unknown>;
  on(event: "pageerror", listener: (error: Error) => void): void;
  on(
    event: "console",
    listener: (message: { type(): string; text(): string }) => void,
  ): void;
  on(
    event: "requestfailed",
    listener: (request: { url(): string }) => void,
  ): void;
  clock: {
    install(options: { time: Date }): Promise<void>;
    runFor(milliseconds: number): Promise<void>;
  };
};

export type BrowserContext = {
  tracing: {
    start(options: {
      screenshots: boolean;
      snapshots: boolean;
      sources: boolean;
    }): Promise<void>;
    stop(options?: { path?: string }): Promise<void>;
  };
  newPage(): Promise<Page>;
  close(): Promise<void>;
  addCookies(
    cookies: {
      name: string;
      value: string;
      url: string;
      httpOnly: boolean;
      secure: boolean;
      sameSite: "Lax";
    }[],
  ): Promise<void>;
  addInitScript(script: string): Promise<void>;
  grantPermissions(
    permissions: string[],
    options?: { origin: string },
  ): Promise<void>;
};

export type Browser = {
  newContext(options: {
    viewport: { width: number; height: number };
    locale: string;
    timezoneId: string;
  }): Promise<BrowserContext>;
  close(): Promise<void>;
};

type Driver = {
  chromium: {
    launch(options: {
      executablePath: string;
      headless: boolean;
      args: string[];
    }): Promise<Browser>;
  };
};

function driver(value: unknown): value is Driver {
  return (
    fields(value) &&
    fields(value.chromium) &&
    typeof value.chromium.launch === "function"
  );
}

export async function launchBrowser(): Promise<Browser> {
  const path = process.env.CIPROXY_PLAYWRIGHT_PATH;
  if (!path)
    throw new Error("Set CIPROXY_PLAYWRIGHT_PATH to playwright-core/index.mjs");
  const loaded: unknown = await import(pathToFileURL(path).href);
  if (!driver(loaded))
    throw new Error("The browser driver has no Chromium launcher");
  return loaded.chromium.launch({
    executablePath: process.env.CIPROXY_CHROME_PATH ?? "/usr/bin/google-chrome",
    headless: true,
    args: ["--no-sandbox", "--disable-dev-shm-usage"],
  });
}
