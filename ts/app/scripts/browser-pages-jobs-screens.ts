import { jobHref, jobsHref } from "../src/view/jobs-links.ts";
import { workspaceId } from "./fixtures-core.ts";
import { fixtureJobIds } from "./fixtures-jobs.ts";
import { screenRuns, type PageScreen } from "./browser-pages-screens.ts";
import type { Page } from "./browser-driver.ts";
import { insideRegion, separateRegions } from "./browser-pages-helpers.ts";

export async function visibleJobDetails(
  page: Page,
  hasRunner = true,
): Promise<void> {
  const details = page.getByRole("region", { name: "Details", exact: true });
  for (const name of hasRunner ? ["Label", "VM size"] : ["Label"]) {
    await insideRegion(details, details.getByText(name, { exact: true }));
  }
  await separateRegions(
    details,
    page.getByRole("region", { name: "Where it ran", exact: true }),
  );
}

export function jobsScreenRuns(origin: string) {
  const live = jobsHref(workspaceId, "live");
  const history = jobsHref(workspaceId, "history");
  const detail = jobHref(workspaceId, fixtureJobIds.machine);
  const screens: PageScreen[] = [
    {
      name: "jobs-live",
      scenario: "m21-jobs-live",
      path: live,
      text: "Build and test",
    },
    {
      name: "jobs-quiet",
      scenario: "m21-jobs-quiet",
      path: live,
      heading: "2 warm runners are ready",
    },
    {
      name: "jobs-waiting-unknown",
      scenario: "m21-jobs-waiting-unknown",
      path: live,
      text: "The GitHub App is suspended on northstar-labs",
    },
    {
      name: "jobs-nothing-fits",
      scenario: "m21-jobs-nothing-fits",
      path: live,
      text: "1 job with the label ciproxy-large waits",
    },
    {
      name: "jobs-fit-unknown",
      scenario: "m21-jobs-fit-unknown",
      path: live,
      text: /Some machine or Boat facts are not available/,
    },
    {
      name: "jobs-spend-limit",
      scenario: "m21-jobs-spend-limit",
      path: live,
      text: /Boat reached its \$100 limit for November/,
    },
    {
      name: "jobs-boat-refused",
      scenario: "m21-jobs-boat-refused",
      path: live,
      text: "Boat refused new sandboxes",
    },
    {
      name: "jobs-overflow-unknown",
      scenario: "m21-jobs-overflow-unknown",
      path: live,
      text: "Waiting build",
    },
    {
      name: "jobs-history",
      scenario: "m21-jobs-history",
      path: history,
      role: { role: "link", name: "Legacy request" },
    },
    {
      name: "jobs-history-empty",
      scenario: "m21-jobs-history-empty",
      path: history,
      heading: "Finished jobs show here",
    },
    {
      name: "jobs-history-no-match",
      scenario: "m21-jobs-history",
      path: history + "&search=fixture-no-match",
      heading: "No finished jobs match",
    },
    {
      name: "jobs-live-error",
      scenario: "m21-jobs-live-error",
      path: live,
      heading: "Unable to load jobs",
      statuses: [503],
    },
    {
      name: "jobs-history-error",
      scenario: "m21-jobs-history-error",
      path: history,
      heading: "Unable to load finished jobs",
      statuses: [503],
    },
    {
      name: "job-machine",
      scenario: "m21-jobs-detail",
      path: detail,
      heading: "Build and test",
      text: "Counts as 1 start",
    },
    {
      name: "job-boat",
      scenario: "m21-jobs-detail",
      path: jobHref(workspaceId, fixtureJobIds.boat),
      heading: "Browser checks",
      text: "Boat bills this size.",
    },
    {
      name: "job-unknown-request",
      scenario: "m21-jobs-detail",
      path: jobHref(workspaceId, fixtureJobIds.unknown),
      heading: "Legacy request",
      text: "Requested size unknown",
    },
    {
      name: "job-running-unknown-request",
      scenario: "m21-jobs-request-unknown",
      path: jobHref(workspaceId, fixtureJobIds.running),
      heading: "Build and test",
      text: "Requested size unknown",
    },
    {
      name: "job-cancelled",
      scenario: "m21-jobs-detail",
      path: jobHref(workspaceId, fixtureJobIds.cancelled),
      heading: "Cancelled before a runner",
      text: "No ciproxy runner took this job, so it is not a start.",
    },
    {
      name: "job-unknown-timing",
      scenario: "m21-jobs-detail",
      path: jobHref(workspaceId, fixtureJobIds.untimed),
      heading: "Completed with unknown timing",
      text: "Counts as 1 start",
    },
    {
      name: "job-not-found",
      scenario: "m21-jobs-detail-not-found",
      path: detail,
      heading: "This job is not in your history",
      statuses: [404],
    },
    {
      name: "job-error",
      scenario: "m21-jobs-detail-error",
      path: detail,
      heading: "Unable to load the job",
      statuses: [503],
    },
    {
      name: "jobs-viewer",
      scenario: "m21-viewer",
      path: live,
      text: "Build and test",
    },
    {
      name: "jobs-billing",
      scenario: "m21-billing",
      path: live,
      text: "Build and test",
    },
  ];
  return screenRuns(
    origin,
    screens.map((screen) =>
      screen.name.startsWith("job-") &&
      screen.name !== "job-not-found" &&
      screen.name !== "job-error"
        ? {
            ...screen,
            check: (page) =>
              visibleJobDetails(page, screen.name !== "job-cancelled"),
          }
        : screen,
    ),
  );
}
