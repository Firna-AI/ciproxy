// The job model of the fixtures and the helpers that build job rows. The
// jobs come from both organizations of the workspace, and a row names the
// organization of its repository. A job runs on an own machine, in one of
// its slots, or on a provider, which a row names.

import { labsOrg, org } from "./org.ts";
import type { ProviderKey } from "./providers.ts";

export type JobState =
  "running" | "waiting" | "passed" | "failed" | "cancelled";

export type Where =
  | { kind: "own"; machine: string; slot: number }
  | { kind: "provider"; provider: ProviderKey }
  | { kind: "none" };

/** The repositories of the jobs: two of mokly-ai and one of mokly-labs. */
export type Repo = "mokly" | "mokly-cloud" | "eval-runner";

const repoOrganization: Readonly<Record<Repo, string>> = {
  mokly: org.login,
  "mokly-cloud": org.login,
  "eval-runner": labsOrg.login,
};

/** "mokly-ai/mokly", the repository of a job with its organization. */
export function repositoryName(repo: Repo): string {
  return `${repoOrganization[repo]}/${repo}`;
}

export type Job = {
  id: string;
  name: string;
  workflow: string;
  repo: Repo;
  ref: string;
  state: JobState;
  where: Where;
  /** Running for, waiting for, or the duration of a finished job. */
  time: string;
  /** The label in runs-on, which gives the job its size. */
  label: string;
  /** Finished jobs: the time the job waited for a runner. */
  waited?: string;
  /** Finished jobs: the clock time it finished. */
  finished?: string;
};

/** The label of a job whose row names no other: the default label. */
export const defaultLabel = "ciproxy";

export const own = (index: number, slot: number): Where => ({
  kind: "own",
  machine: `hetzner-ax42-${index}`,
  slot,
});
export const onBoat: Where = { kind: "provider", provider: "boat" };
export const onSecond: Where = { kind: "provider", provider: "second" };
export const none: Where = { kind: "none" };

/** The provider that runs or ran a job, when a provider does. */
export function providerOf(where: Where): ProviderKey | undefined {
  return where.kind === "provider" ? where.provider : undefined;
}

export type Row = [
  string,
  string,
  Job["repo"],
  string,
  JobState,
  Where,
  string,
  string?,
];

/**
 * Jobs from compact rows: name, workflow, repo, ref, state, where, time, and
 * the label when it is not the default label.
 */
export function rows(prefix: string, list: readonly Row[]): Job[] {
  return list.map(
    (
      [name, workflow, repo, ref, state, where, time, label = defaultLabel],
      i,
    ) => ({
      id: `${prefix}-${i + 1}`,
      name,
      workflow,
      repo,
      ref,
      state,
      where,
      time,
      label,
    }),
  );
}
