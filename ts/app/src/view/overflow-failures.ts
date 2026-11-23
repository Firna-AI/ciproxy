// What the person reads when a change of a provider did not finish. A key
// that the provider refused belongs to the key field, and a limit that the
// API refused to the limit field. Any other failure names the action and the
// provider, and one way to recover. The API saves nothing on a refused
// change, so each recovery repeats it.

import type { ApiFailure } from "../api/client.ts";
import type { OverflowProvider } from "../api/schema.ts";
import type { FailureCopy } from "./claim.ts";
import { providerCopy, providerName } from "./providers.ts";

export const limitInvalid = "Enter a limit above $0, such as 200.";

/** The error of an empty key field. */
export function keyMissing(provider: OverflowProvider): string {
  return `Enter the API key of your ${providerName(provider)} account.`;
}

export type FieldProblem =
  | { field: "key" | "limit"; message: string }
  | { field: null; copy: FailureCopy };

/** The problem code, with a refused key only of this provider. */
function code(failure: ApiFailure, provider: OverflowProvider): string | null {
  if (failure.kind !== "problem") return null;
  const refused = failure.problem.code === "provider_key_rejected";
  if (refused && failure.problem.provider !== provider) return null;
  return failure.problem.code;
}

function retry(failure: ApiFailure, action: string): string {
  return failure.kind === "network"
    ? `Check the connection, then try ${action} again.`
    : `Try ${action} again.`;
}

/** A key check that did not finish: a refused key, or another failure. */
export function checkProblem(
  failure: ApiFailure,
  provider: OverflowProvider,
): FieldProblem {
  const title = "Unable to check the key";
  switch (code(failure, provider)) {
    case "provider_key_rejected":
      return { field: "key", message: providerCopy[provider].keyError };
    case "conflict":
      return {
        field: null,
        copy: { title, body: "Reload the page, then try Check key again." },
      };
    default:
      return {
        field: null,
        copy: { title, body: retry(failure, "Check key") },
      };
  }
}

/** A connection that did not finish: a refused key, a limit, or another. */
export function connectProblem(
  failure: ApiFailure,
  provider: OverflowProvider,
): FieldProblem {
  const action = `Connect ${providerName(provider)}`;
  const title = `Unable to connect ${providerName(provider)}`;
  switch (code(failure, provider)) {
    case "provider_key_rejected":
      return { field: "key", message: providerCopy[provider].keyError };
    case "invalid_request":
      return {
        field: "limit",
        message: `Check the limit, then try ${action} again.`,
      };
    case "conflict":
      return {
        field: null,
        copy: { title, body: `Reload the page, then try ${action} again.` },
      };
    default:
      return { field: null, copy: { title, body: retry(failure, action) } };
  }
}

/** A save of the limit of a provider that did not finish. */
export function limitSaveProblem(
  failure: ApiFailure,
  provider: OverflowProvider,
): FieldProblem {
  if (code(failure, provider) === "invalid_request") {
    return {
      field: "limit",
      message: "Check the limit, then try Save limit again.",
    };
  }
  return {
    field: null,
    copy: {
      title: "Unable to save the limit",
      body: retry(failure, "Save limit"),
    },
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

export function disconnectFailure(
  failure: ApiFailure,
  provider: OverflowProvider,
): FailureCopy {
  return {
    title: `Unable to disconnect ${providerName(provider)}`,
    body: retry(failure, "Disconnect"),
  };
}
