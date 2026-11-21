import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobDetailScreen } from "../../../../screens/jobs/job-detail.tsx";
import { JobReadScreen } from "../../../../screens/jobs/job-read-states.tsx";

export const loading = defineScreen({
  slug: "loading",
  title: "While the job loads",
  description: "Skeleton bars keep the layout of the two panels.",
  relatedDocs: related("jobRouting"),
  address: orgAddress("jobs/48213977"),
  render: (_props, context) => (
    <JobReadScreen viewport={context.viewport} state="loading" />
  ),
});

export const error = defineScreen({
  slug: "error",
  title: "When the job fails to load",
  description: "The shared failed-read state with Try again.",
  relatedDocs: related("jobRouting"),
  address: orgAddress("jobs/48213977"),
  render: (_props, context) => (
    <JobReadScreen viewport={context.viewport} state="error" />
  ),
});

export const notFound = defineScreen({
  slug: "not-found",
  title: "The job is not in the history",
  description:
    "An old link opens a job that the history of the plan no longer shows, or a job of another workspace. The API answers that it has no such job. Open History leads back.",
  rationale:
    "security.md: each page shows only the jobs of the job history of the tier. The page names both reasons, because the API does not tell them apart.",
  relatedDocs: related("security", "billing"),
  address: orgAddress("jobs/48100001"),
  render: (_props, context) => (
    <JobReadScreen viewport={context.viewport} state="not-found" />
  ),
});

export const noRunner = defineScreen({
  slug: "no-runner",
  title: "A job that no runner took",
  description:
    "e2e (webkit) 2/2 was cancelled on GitHub before a ciproxy runner took it. The page names no place, no VM size, no wait, and no start. History opens this page from its row.",
  rationale:
    "metering.md: a job that no runner takes is not a start, for example a job that someone cancels while it waits. The API sends no place and no actual size for it.",
  relatedDocs: related("metering", "jobRouting"),
  address: orgAddress("jobs/48213921"),
  render: (_props, context) => (
    <JobDetailScreen viewport={context.viewport} state="no-runner" />
  ),
});

export const unknownFacts = defineScreen({
  slug: "unknown-facts",
  title: "A job with unknown facts",
  description:
    "deploy preview ran on hetzner-ax42-3. GitHub reports it as passed with no finish time, ciproxy has no saved request and no report of its VM, and the machine cache sent no counts. Each such value says Unknown, never 0 or the current label size.",
  rationale:
    "The API keeps unknown values apart from zero: the duration, the VM size, the request, and the cache facts can each be absent.",
  relatedDocs: related("jobRouting", "cache", "labels"),
  address: orgAddress("jobs/48213931"),
  render: (_props, context) => (
    <JobDetailScreen viewport={context.viewport} state="unknown-facts" />
  ),
});
