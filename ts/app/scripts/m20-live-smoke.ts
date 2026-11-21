import assert from "node:assert/strict";

import type { components } from "../../api/index.ts";
import { createApiClient } from "../src/api/client.ts";
import { createEndpoints } from "../src/api/endpoints.ts";
import { createMachineEndpoints } from "../src/api/machine-endpoints.ts";
import { createSettingsEndpoints } from "../src/api/settings-endpoints.ts";
import { createBuildGate } from "../src/state/build-gate.ts";
import { liveBrowser, type LiveCheck } from "./m20-live-browser.ts";
import { loadCredentials } from "./m20-live-credentials.ts";

type Schemas = components["schemas"];
const results: { name: string; status: "pass" | "fail" | "skip" }[] = [];
const check: LiveCheck = async (name, run) => {
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
  const config = await loadCredentials(
    process.env.CIPROXY_DASHBOARD_CREDENTIALS_FILE,
  );
  if (config === undefined) {
    process.stderr.write(
      "SKIP live dashboard: private credentials are absent\n",
    );
    process.exitCode = 2;
  } else {
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
    const access = createEndpoints(client);
    const machines = createMachineEndpoints(client);
    const settings = createSettingsEndpoints(client);
    let machine: Schemas["MachineView"] | undefined;
    let organization: Schemas["OrganizationView"] | undefined;
    let label: Schemas["LabelView"] | undefined;
    await check("session access and current CSRF", async () => {
      const result = await access.session();
      if (!result.ok) throw new Error("Session unavailable");
      assert.equal(result.value.state, "ready");
      assert.equal(
        result.value.workspaces.find((item) => item.id === config.workspace_id)
          ?.role,
        "admin",
      );
      csrf = result.value.csrf_token;
    });
    await check("typed Machines and overflow summary", async () => {
      const result = await machines.machines(config.workspace_id);
      if (!result.ok) throw new Error("Machines unavailable");
      machine = result.value[0];
      assert.equal(
        (await machines.overflowSummary(config.workspace_id)).ok,
        true,
      );
    });
    if (!machine) {
      results.push({ name: "machine detail", status: "skip" });
      process.stdout.write("SKIP machine detail: no registered machine\n");
    }
    await check("typed Settings labels and organizations", async () => {
      const result = await settings.settings(config.workspace_id);
      if (!result.ok) throw new Error("Settings unavailable");
      label = result.value.labels[0];
      organization = result.value.organizations.find(
        (item) => item.state !== "removed",
      );
      assert.notEqual(label, undefined);
    });
    await check("typed Members roles and access requests", async () => {
      assert.equal((await settings.members(config.workspace_id)).ok, true);
    });
    await check("preview one existing label", async () => {
      if (!label) throw new Error("Label unavailable");
      assert.equal(
        (await settings.previewLabel(config.workspace_id, label.label)).ok,
        true,
      );
    });
    if (organization) {
      await check("typed repository access", async () => {
        if (!organization) throw new Error("Organization unavailable");
        assert.equal(
          (await settings.repositories(config.workspace_id, organization.id))
            .ok,
          true,
        );
      });
    } else {
      results.push({ name: "repository access", status: "skip" });
      process.stdout.write(
        "SKIP repository access: no installed organization\n",
      );
    }
    await check("create and revoke one install token", async () => {
      const issued = await access.issueInstallCommand(config.workspace_id);
      if (!issued.ok) throw new Error("Install command unavailable");
      // Revoke before checking or reporting the secret-bearing body.
      const revoked = await access.revokeInstallCommand(
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
    await liveBrowser(config, check, machine, organization);
  }
} catch {
  process.exitCode = 1;
  if (!results.some((result) => result.status === "fail")) {
    results.push({
      name: "private credentials or browser setup",
      status: "fail",
    });
    process.stderr.write("FAIL private credentials or browser setup\n");
  }
}
if (results.length > 0) {
  const count = (status: "pass" | "fail" | "skip") =>
    results.filter((result) => result.status === status).length;
  process.stdout.write(
    `Live checks ${results.length}; pass ${count("pass")}; fail ${count("fail")}; skip ${count("skip")}\n`,
  );
}
