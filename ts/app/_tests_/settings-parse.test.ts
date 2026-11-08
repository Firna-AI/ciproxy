import assert from "node:assert/strict";
import test from "node:test";

import {
  parseMembers,
  parseRepositories,
  parseSettings,
} from "../src/api/settings-parse.ts";
import {
  label,
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
  const counted = label("a", {}, {}, "0");
  const unknown = label(
    "b",
    {},
    { boat: { state: "unavailable" }, boat_connected: null },
    null,
  );
  const noBoat = label(
    "c",
    {},
    { boat: { state: "unavailable" }, boat_connected: false },
    "0",
  );
  const labels = settingsView({ labels: [counted, unknown, noBoat] });
  assert.deepEqual(
    parseSettings(JSON.parse(JSON.stringify(labels)))?.labels.map((item) => [
      item.jobs_last_30_days,
      item.fit.boat.state,
      item.fit.boat_connected,
    ]),
    [
      ["0", "fitting", true],
      [null, "unavailable", null],
      ["0", "unavailable", false],
    ],
    "a zero and a missing account stay apart from what ciproxy does not know",
  );
  for (const fit of [
    { ...counted.fit, placement: "somewhere" },
    { ...counted.fit, boat: { state: "fitting", refused: false } },
  ]) {
    assert.equal(
      parseSettings({ ...labels, labels: [{ ...counted, fit }] }),
      undefined,
      "an unknown placement or a Boat size without its size rejects the body",
    );
  }
});

test("the members reader accepts a request and rejects an unknown role", () => {
  const value = membersView([
    member("101", "mira", "admin", "2026-09-01T09:00:00Z", {
      name: "Mira Chen",
      granted_at: "2026-09-04T15:30:00Z",
    }),
    member("106", null, null, "2026-11-19T08:00:00Z", {
      request_organizations: [
        { id: "40000000-0000-4000-8000-000000000001", login: "mokly-ai" },
      ],
    }),
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
