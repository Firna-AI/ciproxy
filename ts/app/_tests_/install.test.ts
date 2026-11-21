import assert from "node:assert/strict";
import test from "node:test";

import type { InstallLink } from "../src/api/schema.ts";
import { session } from "../scripts/fixtures-core.ts";
import { appWith, deferred, response } from "./support.ts";

test("installation starts with CSRF and opens only the returned URL after saving its entry", async () => {
  const saved = deferred<Response>();
  const data = session();
  const link: InstallLink = {
    attempt_id: "saved-attempt",
    url: "https://github.example/apps/ciproxy/installations/new?state=saved",
  };
  const { app, calls, documents } = appWith(response(data), saved.promise);
  await app.session.ensure();
  const started = app.startInstallation();
  assert.deepEqual(documents, []);
  assert.equal(app.installStarted(), false);
  assert.deepEqual(
    calls.map(({ url, init }) => [url, init.method]),
    [
      ["/api/session", "GET"],
      ["/api/installation-links/start", "POST"],
    ],
  );
  assert.equal(
    new Headers(calls[1]?.init.headers).get("x-ciproxy-csrf"),
    data.csrf_token,
  );
  saved.resolve(response(link));
  assert.deepEqual(await started, { ok: true, value: link });
  assert.deepEqual(documents, [link.url]);
  assert.equal(app.installStarted(), true);
});

test("a refused installation start keeps the document and its install marker unchanged", async () => {
  const { app, documents } = appWith(response(session()), new Error("offline"));
  await app.session.ensure();
  assert.deepEqual(await app.startInstallation(), {
    ok: false,
    failure: { kind: "network" },
  });
  assert.deepEqual(documents, []);
  assert.equal(app.installStarted(), false);
});
