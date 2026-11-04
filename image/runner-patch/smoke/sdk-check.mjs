#!/usr/bin/env node
/** One-time conformance check of the installed official @actions/cache client. */

import assert from "node:assert/strict";
import { createHash, randomBytes, randomUUID } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { join, resolve } from "node:path";

const require = createRequire(import.meta.url);
const modulePath = process.argv[2];
const proxy = process.env.CIPROXY_SMOKE_PROXY;
const context = process.env.CIPROXY_SMOKE_DIR;
assert.ok(
  modulePath && proxy && context,
  "set CIPROXY_SMOKE_PROXY, CIPROXY_SMOKE_DIR, and pass the installed cache module path",
);
const cache = require(resolve(modulePath));
await mkdir(context, { recursive: true });
const directory = await mkdtemp(join(context, "sdk-"));
const filename = join(directory, "payload");
process.env.RUNNER_TEMP = directory;
process.env.ACTIONS_RUNTIME_TOKEN =
  process.env.ACTIONS_RUNTIME_TOKEN || "fake-runtime-token";
process.env.ACTIONS_CACHE_SERVICE_V2 = "true";
process.env.ACTIONS_RESULTS_URL = `${proxy}/`;
process.env.ACTIONS_CACHE_URL = `${proxy}/`;
const size = Number(process.env.CIPROXY_SMOKE_BYTES || 52428800);
assert.ok(Number.isSafeInteger(size) && size > 0);
const payload = randomBytes(size);
const expected = createHash("sha256").update(payload).digest("hex");
const key = `ciproxy-smoke-${randomUUID()}`;
try {
  await writeFile(filename, payload);
  const id = await cache.saveCache([filename], key);
  assert.ok(id > 0, "the official client saved its archive");
  await rm(filename);
  assert.equal(
    await cache.restoreCache([filename], key, [], { lookupOnly: true }),
    key,
  );
  await assert.rejects(readFile(filename), { code: "ENOENT" });
  assert.equal(
    await cache.restoreCache([filename], `${key}-missing`, [key]),
    key,
  );
  assert.equal(
    createHash("sha256")
      .update(await readFile(filename))
      .digest("hex"),
    expected,
  );
  console.log(
    `PASS: official @actions/cache save, lookup-only, prefix restore, ${size} payload bytes`,
  );
} finally {
  await rm(directory, { recursive: true, force: true });
}
