// What the person reads when a claim of the Admin role did not finish. The
// API gives the same answer for a claim that its rule refuses, a refused
// CSRF token, and other refusals, so the copy names no cause that the app
// cannot know. A claim with no answer at all names the connection.

import type { ApiFailure } from "../api/client.ts";

export type FailureCopy = { title: string; body: string };

export function claimFailure(failure: ApiFailure): FailureCopy {
  return {
    title: "Unable to claim the Admin role",
    body:
      failure.kind === "network"
        ? "Check the connection, then try again."
        : "ciproxy could not complete the claim. Try again.",
  };
}
