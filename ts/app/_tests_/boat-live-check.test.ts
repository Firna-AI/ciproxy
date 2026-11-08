import assert from "node:assert/strict";
import test from "node:test";

import type { components } from "../../api/index.ts";
import {
  runBoatLiveCheck,
  type BoatLiveDependencies,
} from "../scripts/boat-live-check.ts";
import { session, workspace } from "../scripts/fixtures-core.ts";

const privateMarker = "made-up-private-value";
const credentialPath = "/private/boat-live-credentials.json";
const credentials = {
  base_url: "https://boat-live.example",
  workspace_id: "test / workspace",
  session_cookie: privateMarker,
  csrf_token: "fixture-csrf-current",
};

const readySession = () =>
  session({ workspaces: [workspace({ id: credentials.workspace_id })] });

const settings = {
  account: {
    connected_at: "2026-11-07T00:00:00Z",
    name: privateMarker,
    rejected_at: null,
  },
  available: true,
  enabled: false,
  next_month_at: "2026-12-01T00:00:00Z",
  spend: null,
  spend_limit_cents: "10000",
  wait_seconds: 30,
} satisfies components["schemas"]["OverflowSettings"];

function response(value: unknown, build = "recorded-build"): Response {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { "Content-Type": "application/json", "x-ciproxy-build": build },
  });
}

function subject(
  replies: (Response | Error)[] = [],
  file: string | Error = JSON.stringify(credentials),
) {
  const calls: { url: string; init: RequestInit }[] = [];
  const files: string[] = [];
  const output: string[] = [];
  const dependencies: BoatLiveDependencies = {
    readCredentials: async (path) => {
      files.push(path);
      if (file instanceof Error) throw file;
      return file;
    },
    transport: async (url, init) => {
      calls.push({ url, init });
      const next = replies.shift();
      if (next instanceof Error) throw next;
      if (next === undefined) throw new Error("Unexpected request");
      return next;
    },
    write: (text) => {
      output.push(text);
    },
  };
  return { dependencies, calls, files, output };
}

function assertPrivateOutput(output: string[]): void {
  const text = output.join("");
  for (const value of [
    privateMarker,
    credentialPath,
    ...Object.values(credentials),
  ]) {
    assert.equal(text.includes(value), false);
  }
}

test("Boat live help needs no credentials and makes no requests", async () => {
  const check = subject();
  assert.equal(
    await runBoatLiveCheck(["--help"], undefined, check.dependencies),
    0,
  );
  assert.deepEqual(check.files, []);
  assert.deepEqual(check.calls, []);
});

test("Boat live check skips a missing credential path without a request", async () => {
  const check = subject();
  assert.equal(await runBoatLiveCheck([], undefined, check.dependencies), 2);
  assert.match(check.output.join(""), /^SKIP\b/);
  assert.deepEqual(check.files, []);
  assert.deepEqual(check.calls, []);
});

test("Boat live check refuses write commands before reading credentials", async () => {
  for (const args of [["connect"], ["connect", privateMarker]]) {
    const check = subject();
    assert.equal(
      await runBoatLiveCheck(args, credentialPath, check.dependencies),
      1,
    );
    assert.deepEqual(check.files, []);
    assert.deepEqual(check.calls, []);
    assert.match(check.output.join(""), /^FAIL\b/);
    assertPrivateOutput(check.output);
  }
});

test("Boat live check reads typed settings with private session headers only", async () => {
  const check = subject([response(readySession()), response(settings)]);
  assert.equal(
    await runBoatLiveCheck([], credentialPath, check.dependencies),
    0,
  );
  assert.deepEqual(check.files, [credentialPath]);
  assert.deepEqual(
    check.calls.map(({ url, init }) => ({
      url,
      method: init.method,
      cookie: new Headers(init.headers).get("Cookie"),
      csrf: new Headers(init.headers).get("x-ciproxy-csrf"),
      origin: new Headers(init.headers).get("Origin"),
      redirect: init.redirect,
      cache: init.cache,
      body: init.body,
    })),
    [
      {
        url: "https://boat-live.example/api/session",
        method: "GET",
        cookie: "__Host-ciproxy=made-up-private-value",
        csrf: "fixture-csrf-current",
        origin: "https://boat-live.example",
        redirect: "error",
        cache: "no-store",
        body: undefined,
      },
      {
        url: "https://boat-live.example/api/workspaces/test%20%2F%20workspace/overflow",
        method: "GET",
        cookie: "__Host-ciproxy=made-up-private-value",
        csrf: "fixture-csrf-current",
        origin: "https://boat-live.example",
        redirect: "error",
        cache: "no-store",
        body: undefined,
      },
    ],
  );
  assert.match(check.output.join(""), /^PASS\b/);
  assertPrivateOutput(check.output);
});

test("Boat live check stops before settings for stale CSRF or no Admin role", async () => {
  const sessions = [
    { ...readySession(), csrf_token: privateMarker },
    session({
      workspaces: [workspace({ id: credentials.workspace_id, role: "viewer" })],
    }),
  ];
  for (const value of sessions) {
    const check = subject([response(value)]);
    assert.equal(
      await runBoatLiveCheck(["check"], credentialPath, check.dependencies),
      1,
    );
    assert.equal(check.calls.length, 1);
    assert.match(check.output.join(""), /\bFAIL\b/);
    assertPrivateOutput(check.output);
  }
});

test("Boat live check keeps malformed credential files and read errors private", async () => {
  for (const file of [
    `{"session_cookie":"${privateMarker}",`,
    JSON.stringify({
      ...credentials,
      base_url: `https://${privateMarker}@host`,
    }),
    new Error(`${credentialPath}: ${privateMarker}`),
  ]) {
    const check = subject([], file);
    assert.equal(
      await runBoatLiveCheck([], credentialPath, check.dependencies),
      1,
    );
    assert.deepEqual(check.calls, []);
    assert.match(check.output.join(""), /^FAIL\b/);
    assertPrivateOutput(check.output);
  }
});

test("Boat live check hides HTTP bodies and network or response errors", async () => {
  const failures = [
    new Response(privateMarker, { status: 403 }),
    new Error(privateMarker),
    response({ account: privateMarker }),
    new Response(`{"account":"${privateMarker}`, {
      status: 200,
      headers: { "x-ciproxy-build": "recorded-build" },
    }),
  ];
  for (const failure of failures) {
    const check = subject([response(readySession()), failure]);
    assert.equal(
      await runBoatLiveCheck([], credentialPath, check.dependencies),
      1,
    );
    assert.equal(check.calls.length, 2);
    assert.match(check.output.join(""), /\bFAIL\b/);
    assertPrivateOutput(check.output);
  }
});

test("Boat live check refuses missing or changed API build headers", async () => {
  for (const build of ["", "new-build"]) {
    const check = subject([
      response(readySession()),
      response(settings, build),
    ]);
    assert.equal(
      await runBoatLiveCheck([], credentialPath, check.dependencies),
      1,
    );
    assert.equal(check.calls.length, 2);
    assert.match(check.output.join(""), /\bFAIL\b/);
    assertPrivateOutput(check.output);
  }
});
