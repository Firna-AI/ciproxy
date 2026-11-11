// The dot colour of each machine state, as the mockups give it.

import type { StatusTone } from "@ciproxy/tokens";

import type { MachineStatus } from "../../view/machines.ts";

export const statusTones: Readonly<Record<MachineStatus, StatusTone>> = {
  healthy: "ok",
  draining: "warn",
  offline: "err",
  "no-slots": "err",
};
