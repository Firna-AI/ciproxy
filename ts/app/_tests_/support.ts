import assert from "node:assert/strict";

import type { Transport } from "../src/api/client.ts";
import { createAppServices } from "../src/services/services.ts";
import type { Timers, Visibility } from "../src/state/poller.ts";
import { memoryStorage } from "../src/state/storage.ts";
import { fixtureTime } from "../scripts/fixtures-core.ts";

export function deferred<T>() {
  let resolve: (value: T) => void = () => {
    throw new Error("Uninitialized deferred");
  };
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

export function response(
  body: unknown,
  status = 200,
  build = "fixture-build-a",
): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json", "x-ciproxy-build": build },
  });
}

export type Reply = Response | Promise<Response> | Error;

export function transportQueue(...replies: Reply[]) {
  const calls: { url: string; init: RequestInit }[] = [];
  const transport: Transport = async (url, init) => {
    calls.push({ url, init });
    const reply = replies.shift();
    assert.notEqual(
      reply,
      undefined,
      `Unexpected API call: ${init.method} ${url}`,
    );
    if (reply instanceof Error) throw reply;
    if (reply === undefined) throw new Error("Missing reply");
    return reply;
  };
  return { transport, calls };
}

export function manualSchedule() {
  let hidden = false;
  const callbacks = new Set<() => void>();
  const listeners = new Set<() => void>();
  const delays: number[] = [];
  const timers: Timers = {
    schedule(callback, delay) {
      delays.push(delay);
      callbacks.add(callback);
      return () => {
        callbacks.delete(callback);
      };
    },
  };
  const visibility: Visibility = {
    hidden: () => hidden,
    watch(listener) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
  return {
    timers,
    visibility,
    delays,
    pending: () => callbacks.size,
    fire() {
      const next = callbacks.values().next().value;
      assert.notEqual(next, undefined, "Expected a scheduled task");
      if (next !== undefined) {
        callbacks.delete(next);
        next();
      }
    },
    hide(value: boolean) {
      hidden = value;
      for (const listener of [...listeners]) listener();
    },
  };
}

export async function settled(): Promise<void> {
  // Flush the finite promise chain of Response.text and store updates.
  for (let i = 0; i < 20; i += 1) await Promise.resolve();
}

export function appWith(...replies: Reply[]) {
  const queue = transportQueue(...replies);
  const schedule = manualSchedule();
  const documents: string[] = [];
  const routed: string[] = [];
  let now = Date.parse(fixtureTime);
  const app = createAppServices({
    transport: queue.transport,
    location: {
      origin: "https://dashboard.example",
      assign: (href) => {
        documents.push(href);
      },
      replace: (href) => {
        documents.push(href);
      },
      reload: () => {
        documents.push("reload");
      },
    },
    router: {
      push: (href) => {
        routed.push(href);
      },
      replace: (href) => {
        routed.push(href);
      },
    },
    local: memoryStorage(),
    tab: memoryStorage(),
    now: () => now,
    timers: schedule.timers,
    visibility: schedule.visibility,
  });
  return {
    app,
    ...queue,
    schedule,
    documents,
    routed,
    setNow: (value: number) => {
      now = value;
    },
  };
}
