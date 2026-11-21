// The Boat routes of the fixture API: the settings, the policy, the key
// check, and the connection. They act on the Boat fixture of the current
// scenario. The key check saves nothing. The harness keeps no key: it
// records only the field names and the wallet identity of the last
// connection body.
import type { IncomingMessage } from "node:http";

import type { OverflowPolicy, OverflowSettings } from "../src/api/schema.ts";
import { boatSpend, boatWallets, fixtureBoatKey } from "./fixtures-boat.ts";
import { fixtureTime } from "./fixtures-core.ts";
import type { BoatFixture } from "./harness-boat.ts";
import { fields, problem, success } from "./harness-state.ts";

function positiveCents(value: unknown): value is string {
  return typeof value === "string" && /^\d+$/.test(value) && BigInt(value) > 0n;
}

function policy(value: unknown): OverflowPolicy | undefined {
  if (!fields(value)) return undefined;
  const { enabled, wait_seconds, spend_limit_cents } = value;
  if (typeof enabled !== "boolean" || !positiveCents(spend_limit_cents))
    return undefined;
  if (
    typeof wait_seconds !== "number" ||
    ![0, 15, 30, 60, 120].includes(wait_seconds)
  )
    return undefined;
  return { enabled, wait_seconds, spend_limit_cents };
}

/** The counted spend reaches the limit, as the API counts it exactly. */
function reached(settings: OverflowSettings): void {
  const spend = settings.spend ?? null;
  const limit = settings.spend_limit_cents ?? null;
  if (spend === null || limit === null || !spend.bounded) return;
  const counted =
    BigInt(spend.confirmed_cents) + BigInt(spend.unconfirmed_cents);
  spend.limit_reached = counted >= BigInt(limit);
}

type Send = (status: number, body: unknown) => void;

/** Answer a Boat route of an Admin, or return false for another route. */
export async function serveBoat(
  request: IncomingMessage,
  action: string,
  boat: BoatFixture,
  read: (request: IncomingMessage) => Promise<unknown>,
  send: Send,
): Promise<boolean> {
  const method = request.method ?? "GET";
  const unavailable = () => send(503, problem("unavailable", "Try again."));
  const accepted = () => {
    if (boat.failReadAfterChange) boat.nextReadFails = true;
    send(200, success);
  };
  if (action === "overflow" && method === "GET") {
    if (boat.readFails || boat.nextReadFails) {
      boat.nextReadFails = false;
      unavailable();
    } else send(200, boat.settings);
    return true;
  }
  if (action === "overflow" && method === "PUT") {
    const body = policy(await read(request));
    if (body === undefined) send(400, problem("invalid_request", "Check."));
    else if (boat.saveFails) unavailable();
    else {
      Object.assign(boat.settings, body);
      reached(boat.settings);
      accepted();
    }
    return true;
  }
  if (action === "overflow/wallets" && method === "POST") {
    const body = await read(request);
    if (!boat.settings.available) unavailable();
    else if (!fields(body) || typeof body.api_key !== "string")
      send(400, problem("invalid_request", "Check the request."));
    else if (body.api_key !== fixtureBoatKey)
      send(422, problem("boat_key_rejected", "Boat did not accept this key."));
    else send(200, boatWallets());
    return true;
  }
  if (action !== "overflow/connection") return false;
  if (!boat.settings.available || boat.connectFails) {
    unavailable();
    return true;
  }
  if (method === "DELETE") {
    boat.settings = { ...boat.settings, account: null };
    accepted();
    return true;
  }
  if (method !== "POST") return false;
  const body = await read(request);
  if (boat.holdConnection) {
    await new Promise<void>((done) => {
      boat.release = done;
    });
  }
  boat.connectionFields = fields(body) ? Object.keys(body).sort() : [];
  boat.connectionWallet =
    fields(body) && typeof body.wallet === "string" ? body.wallet : null;
  if (!fields(body) || !positiveCents(body.spend_limit_cents)) {
    send(400, problem("invalid_request", "Check the request."));
  } else if (body.api_key !== fixtureBoatKey) {
    send(422, problem("boat_key_rejected", "Boat did not accept this key."));
  } else {
    boat.settings = {
      ...boat.settings,
      account: {
        connected_at: fixtureTime,
        name: "northstar",
        rejected_at: null,
      },
      spend_limit_cents: body.spend_limit_cents,
      spend:
        boat.settings.spend ??
        boatSpend({
          confirmed_cents: "0",
          unconfirmed_cents: "0",
          settled_bound_cents: "0",
        }),
    };
    reached(boat.settings);
    accepted();
  }
  return true;
}
