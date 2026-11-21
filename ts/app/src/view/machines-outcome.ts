// What the machine page says after a change: the reason of a failure, a drain
// or its end that an offline machine gets later, and a cache that the machine
// clears now. A failed Clear cache shows in its dialog, and a failed cost in
// its form, so the page notice leaves them out.

import type { ApiFailure } from "../api/client.ts";
import type { ChangeSnapshot } from "../state/machines-changes.ts";
import type { NoticeTone } from "./page-model.ts";
import { failureCopy, noticeCopy } from "./machines-copy.ts";

/** The safe message of the API, or one recovery step without a message. */
export function failureText(failure: ApiFailure): string {
  if (failure.kind === "problem") return failure.problem.message;
  if (failure.kind === "network") return failureCopy.network;
  return failureCopy.other;
}

export type OutcomeNotice = {
  tone: NoticeTone;
  title: string;
  body: string | null;
};

export function outcomeNotice(
  snapshot: ChangeSnapshot,
  host: string,
): OutcomeNotice | null {
  if (snapshot.status === "failed") {
    const { change } = snapshot;
    if (change.kind === "cost") return null;
    if (change.kind === "command" && change.command === "clear-cache") {
      return null;
    }
    return {
      tone: "danger",
      title:
        change.kind === "command"
          ? failureCopy[change.command](host)
          : failureCopy.warm,
      body: failureText(snapshot.failure),
    };
  }
  if (snapshot.status !== "done" || snapshot.change.kind !== "command") {
    return null;
  }
  const { command } = snapshot.change;
  if (
    snapshot.connected === false &&
    (command === "drain" || command === "resume")
  ) {
    return {
      tone: "info",
      title: noticeCopy.savedTitle,
      body: noticeCopy.savedOffline(host),
    };
  }
  if (command === "clear-cache" && snapshot.connected === true) {
    return { tone: "info", title: noticeCopy.clearing(host), body: null };
  }
  return null;
}
