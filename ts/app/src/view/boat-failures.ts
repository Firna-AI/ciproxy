// What the person reads when a Boat change did not finish. A key that Boat
// refused belongs to the key field, and a limit that the API refused to the
// limit field. Any other failure names the action and one way to recover.
// The API saves nothing on a refused change, so each recovery repeats it.

import type { ApiFailure } from "../api/client.ts";
import type { FailureCopy } from "./claim.ts";

export const keyRefused =
  "Boat did not accept this key. Copy the whole key from the Boat console.";
export const keyMissing = "Enter the API key of your Boat account.";
export const walletMissing = "Choose the wallet that Boat bills.";
export const limitInvalid = "Enter a limit above $0, such as 200.";

export type FieldProblem =
  | { field: "key" | "limit"; message: string }
  | { field: null; copy: FailureCopy };

function code(failure: ApiFailure): string | null {
  return failure.kind === "problem" ? failure.problem.code : null;
}

function retry(failure: ApiFailure, action: string): string {
  return failure.kind === "network"
    ? `Check the connection, then try ${action} again.`
    : `Try ${action} again.`;
}

/** A key check that did not finish: a refused key, or another failure. */
export function checkProblem(failure: ApiFailure): FieldProblem {
  switch (code(failure)) {
    case "boat_key_rejected":
      return { field: "key", message: keyRefused };
    case "conflict":
      return {
        field: null,
        copy: {
          title: "Unable to check the key",
          body: "Reload the page, then try Check key again.",
        },
      };
    default:
      return {
        field: null,
        copy: {
          title: "Unable to check the key",
          body: retry(failure, "Check key"),
        },
      };
  }
}

/** A connection that did not finish: a refused key, a limit, or another. */
export function connectProblem(failure: ApiFailure): FieldProblem {
  switch (code(failure)) {
    case "boat_key_rejected":
      return { field: "key", message: keyRefused };
    case "invalid_request":
      return {
        field: "limit",
        message: "Check the limit, then try Connect Boat again.",
      };
    case "conflict":
      return {
        field: null,
        copy: {
          title: "Unable to connect Boat",
          body: "Reload the page, then try Connect Boat again.",
        },
      };
    default:
      return {
        field: null,
        copy: {
          title: "Unable to connect Boat",
          body: retry(failure, "Connect Boat"),
        },
      };
  }
}

/** A save of the limit that did not finish. */
export function limitSaveProblem(failure: ApiFailure): FieldProblem {
  if (code(failure) === "invalid_request") {
    return { field: "limit", message: "Check the limit, then try Save again." };
  }
  return {
    field: null,
    copy: { title: "Unable to save the limit", body: retry(failure, "Save") },
  };
}

/** A save that the store did not send: the page must read the settings. */
export const unsentSave: FailureCopy = {
  title: "Unable to save the change",
  body: "Reload the page, then try again.",
};

/** A change of the switch or the wait that did not finish. */
export function policySaveFailure(failure: ApiFailure): FailureCopy {
  return {
    title: "Unable to save the change",
    body:
      failure.kind === "network"
        ? "Check the connection, then try again."
        : "Try again.",
  };
}

export function disconnectFailure(failure: ApiFailure): FailureCopy {
  return {
    title: "Unable to disconnect Boat",
    body: retry(failure, "Disconnect"),
  };
}
