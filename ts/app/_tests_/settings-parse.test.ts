import assert from "node:assert/strict";
import test from "node:test";

import {
  parseMembers,
  parseRepositories,
  parseSettings,
} from "../src/api/settings-parse.ts";
import {
  member,
  membersView,
  removed,
  settingsView,
} from "./settings-support.ts";

test("the settings reader keeps a removed organization with its deadline", () => {
  const value = settingsView({
    organizations: [removed(2, "mokly-labs", "2026-12-20T10:00:00Z")],
    open_requests: 2,
  });
  assert.deepEqual(parseSettings(JSON.parse(JSON.stringify(value))), value);
  assert.equal(
    parseSettings({ ...value, open_requests: "2" }),
    undefined,
    "a count must be a whole number",
  );
  assert.equal(
    parseSettings({
      ...value,
      organizations: [{ ...value.organizations[0], state: "gone" }],
    }),
    undefined,
    "an unknown App state rejects the body",
  );
});

test("the members reader accepts a request and rejects an unknown role", () => {
  const value = membersView([
    member("101", "mira", "admin"),
    member("106", null, null),
  ]);
  assert.deepEqual(parseMembers(JSON.parse(JSON.stringify(value))), value);
  assert.equal(
    parseMembers({
      ...value,
      members: [{ ...member("1", "a", "admin"), role: "owner" }],
    }),
    undefined,
  );
});

test("the repositories reader needs the private flag of each repository", () => {
  const repository = {
    id: "7001",
    full_name: "mokly-ai/mokly",
    private: true,
    allowed: true,
  };
  assert.deepEqual(parseRepositories([repository]), [repository]);
  assert.equal(
    parseRepositories([{ ...repository, private: "yes" }]),
    undefined,
  );
});
