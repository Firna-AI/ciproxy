// The generated contract of the machine routes that the shared schema module
// does not name. Rust owns every type here; each alias names one generated
// schema or path parameter and adds nothing.

import type { components, operations } from "../../../api/index.ts";

type Schemas = components["schemas"];

export type BoatAccountView = Schemas["BoatAccountView"];
export type BoatSpendView = Schemas["BoatSpendView"];
export type CommandResult = Schemas["CommandResult"];
export type MachineCostInput = Schemas["MachineCostInput"];
export type OverflowSettings = Schemas["OverflowSettings"];
export type WarmRunnerTarget = Schemas["WarmRunnerTarget"];

/** A command of the machine control route: drain, resume, remove, clear. */
export type MachineCommand =
  operations["machines_command"]["parameters"]["path"]["action"];
