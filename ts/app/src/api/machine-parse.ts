// Readers for machine commands and the workspace overflow summary.

import { boolean, object, ParseError, parser } from "./parse-core.ts";
import type { CommandResult } from "./machine-schema.ts";
import { readOverflowSettings } from "./parse-overflow.ts";

export const parseCommandResult = parser((value): CommandResult => {
  const fields = object(value, "command result");
  return { connected: boolean(fields.connected) };
});

export const parseOverflowSummary = parser((value) => {
  const summary = readOverflowSettings(value);
  if (summary.wait_seconds < 0) throw new ParseError("overflow wait");
  return summary;
});
