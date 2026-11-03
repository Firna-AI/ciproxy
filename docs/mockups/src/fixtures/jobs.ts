// The job model of the fixtures and the helpers that build job rows.

export type JobState =
  "running" | "waiting" | "passed" | "failed" | "cancelled";

export type Where =
  | { kind: "own"; machine: string; slot: number }
  | { kind: "boat" }
  | { kind: "none" };

export type Job = {
  id: string;
  name: string;
  workflow: string;
  repo: "mokly" | "mokly-cloud";
  ref: string;
  state: JobState;
  where: Where;
  /** Running for, waiting for, or the duration of a finished job. */
  time: string;
  /** Finished jobs: the time the job waited for a runner. */
  waited?: string;
  /** Finished jobs: the clock time it finished. */
  finished?: string;
};

export const own = (index: number, slot: number): Where => ({
  kind: "own",
  machine: `hetzner-ax42-${index}`,
  slot,
});
export const onBoat: Where = { kind: "boat" };
export const none: Where = { kind: "none" };

export type Row = [
  string,
  string,
  Job["repo"],
  string,
  JobState,
  Where,
  string,
];

/** Jobs from compact rows: name, workflow, repo, ref, state, where, time. */
export function rows(prefix: string, list: readonly Row[]): Job[] {
  return list.map(([name, workflow, repo, ref, state, where, time], i) => ({
    id: `${prefix}-${i + 1}`,
    name,
    workflow,
    repo,
    ref,
    state,
    where,
    time,
  }));
}
