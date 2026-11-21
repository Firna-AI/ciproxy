// The credential path stays private. This tool prints check names and status only.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import { createApiClient } from "../src/api/client.ts";
import { createEndpoints } from "../src/api/endpoints.ts";
import { createBuildGate } from "../src/state/build-gate.ts";
import { launchBrowser } from "./browser-driver.ts";
import { fields } from "./harness-state.ts";

type Credentials = {
  base_url: string;
  workspace_id: string;
  session_cookie: string;
  csrf_token: string;
};

function credentials(value: unknown): Credentials | undefined {
  if (!fields(value)) return undefined;
  const { base_url, workspace_id, session_cookie, csrf_token } = value;
  if (
    typeof base_url !== "string" ||
    typeof workspace_id !== "string" ||
    typeof session_cookie !== "string" ||
    typeof csrf_token !== "string"
  )
    return undefined;
  if (
    [workspace_id, session_cookie, csrf_token].some(
      (item) => item === "" || /[\r\n]/.test(item),
    )
  )
    return undefined;
  try {
    const url = new URL(base_url);
    if (
      url.protocol !== "https:" ||
      url.username !== "" ||
      url.password !== "" ||
      url.pathname !== "/" ||
      url.search !== "" ||
      url.hash !== ""
    )
      return undefined;
    return { base_url: url.origin, workspace_id, session_cookie, csrf_token };
  } catch {
    return undefined;
  }
}

const file = process.env.CIPROXY_DASHBOARD_CREDENTIALS_FILE;
if (!file) {
  process.stderr.write(
    "SKIP live dashboard: CIPROXY_DASHBOARD_CREDENTIALS_FILE is absent\n",
  );
  process.exitCode = 2;
} else {
  const results: { name: string; status: "pass" | "fail" }[] = [];
  const check = async (name: string, run: () => Promise<void>) => {
    try {
      await run();
      results.push({ name, status: "pass" });
      process.stdout.write(`PASS ${name}\n`);
    } catch {
      results.push({ name, status: "fail" });
      process.stderr.write(`FAIL ${name}\n`);
      throw new Error("Live check failed");
    }
  };
  try {
    const data: unknown = JSON.parse(await readFile(file, "utf8"));
    const config = credentials(data);
    assert.notEqual(config, undefined);
    if (config === undefined) throw new Error("Invalid credential file");
    let csrf = config.csrf_token;
    let missingBuild = false;
    const client = createApiClient({
      gate: createBuildGate(),
      csrf: () => csrf,
      transport: async (path, init) => {
        const headers = new Headers(init.headers);
        headers.set("Cookie", `__Host-ciproxy=${config.session_cookie}`);
        headers.set("Origin", config.base_url);
        const response = await fetch(config.base_url + path, {
          ...init,
          headers,
          redirect: "error",
        });
        if (!response.headers.get("x-ciproxy-build")) missingBuild = true;
        return response;
      },
    });
    const endpoints = createEndpoints(client);
    await check("session access and current CSRF", async () => {
      const result = await endpoints.session();
      assert.equal(result.ok, true);
      if (!result.ok) throw new Error("Session unavailable");
      assert.equal(result.value.state, "ready");
      assert.equal(
        result.value.workspaces.find(
          (workspace) => workspace.id === config.workspace_id,
        )?.role,
        "admin",
      );
      csrf = result.value.csrf_token;
    });
    await check("typed Setup and machines responses", async () => {
      assert.equal((await endpoints.setup(config.workspace_id)).ok, true);
      assert.equal((await endpoints.machines(config.workspace_id)).ok, true);
    });
    await check("create and revoke one install token", async () => {
      const issued = await endpoints.issueInstallCommand(config.workspace_id);
      assert.equal(issued.ok, true);
      if (!issued.ok) throw new Error("Install token unavailable");
      // Revoke before making any assertion about the returned secret-bearing body.
      const revoked = await endpoints.revokeInstallCommand(
        config.workspace_id,
        issued.value.id,
      );
      assert.equal(revoked.ok, true);
      assert.ok(Number.isFinite(Date.parse(issued.value.expires_at)));
      assert.notEqual(issued.value.command, "");
    });
    await check("API build header on every response", async () => {
      assert.equal(missingBuild, false);
    });
    const browser = await launchBrowser();
    try {
      for (const width of [1440, 720, 390]) {
        await check(`live dashboard document reload ${width}`, async () => {
          const context = await browser.newContext({
            viewport: { width, height: 1000 },
            locale: "en-GB",
            timezoneId: "UTC",
          });
          try {
            await context.addCookies([
              {
                name: "__Host-ciproxy",
                value: config.session_cookie,
                url: config.base_url,
                secure: true,
                httpOnly: true,
                sameSite: "Lax",
              },
            ]);
            const page = await context.newPage();
            let failed = false;
            page.on("pageerror", () => {
              failed = true;
            });
            await page.goto(
              `${config.base_url}/jobs?workspace=${encodeURIComponent(config.workspace_id)}`,
              { waitUntil: "domcontentloaded" },
            );
            await page
              .getByRole("heading", { name: "Jobs", exact: true })
              .waitFor({ state: "visible" });
            await page.reload({ waitUntil: "domcontentloaded" });
            await page
              .getByRole("heading", { name: "Jobs", exact: true })
              .waitFor({ state: "visible" });
            assert.equal(failed, false);
            // No screenshot or trace: Setup can show a live install secret.
          } finally {
            await context.close();
          }
        });
      }
    } finally {
      await browser.close();
    }
  } catch {
    process.exitCode = 1;
    if (results.every((result) => result.status === "pass")) {
      results.push({
        name: "private credentials or browser setup",
        status: "fail",
      });
      process.stderr.write("FAIL private credentials or browser setup\n");
    }
  }
  const passed = results.filter((result) => result.status === "pass").length;
  process.stdout.write(
    `Live checks ${results.length}; pass ${passed}; fail ${results.length - passed}; skip 0\n`,
  );
}
