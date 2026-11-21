// The dot colour of each machine state, as the mockups give it. A machine
// that gets its first job image is work in progress, not a fault, so it takes
// the blue dot of a running job.

import type { StatusTone } from "@ciproxy/tokens";

import type { MachineStatus } from "../../view/machines.ts";

export const statusTones: Readonly<Record<MachineStatus, StatusTone>> = {
  healthy: "ok",
  draining: "warn",
  offline: "err",
  "no-slots": "err",
  preparing: "run",
};
