// The generated contract of Jobs: the live queue, the history, and one job.
// Rust owns every type here; the app imports the generated declarations
// relatively. Each alias names one generated type and adds nothing.

import type { components, operations } from "../../../api/index.ts";

type Schemas = components["schemas"];

export type BoatLabelFit = Schemas["BoatLabelFit"];
export type JobCacheCounts = Schemas["JobCacheCounts"];
export type JobCacheSource = Schemas["JobCacheSource"];
export type JobDetailPage = Schemas["JobDetailPage"];
export type JobLabel = Schemas["JobLabel"];
export type JobLocation = Schemas["JobLocation"];
export type JobOverflow = Schemas["JobOverflow"];
export type JobStatus = Schemas["JobStatus"];
export type JobView = Schemas["JobView"];
export type JobVisibility = Schemas["JobVisibility"];
export type JobVmSize = Schemas["JobVmSize"];
export type JobsHistoryPage = Schemas["JobsHistoryPage"];
export type JobsLivePage = Schemas["JobsLivePage"];
export type StartingRunner = Schemas["StartingRunner"];
export type WaitingLabelFit = Schemas["WaitingLabelFit"];

/** The filters of GET jobs/history, as the generated operation names them. */
export type HistoryQuery = NonNullable<
  operations["jobs_history"]["parameters"]["query"]
>;

export type HistoryPlace = NonNullable<HistoryQuery["place"]>;
