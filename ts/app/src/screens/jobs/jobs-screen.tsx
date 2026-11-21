// Jobs: the live queue and the history of finished jobs of one workspace,
// as two tabs of one page. The tab and the History filters live in the
// address, so a reload or a shared link keeps them. Every role reads the
// page, and only the actions of a caller who may act show.

import { useLocalSearchParams } from "expo-router";

import { LinkTabs } from "../../components/link-tabs.tsx";
import { PageHeader } from "../../components/page-state.tsx";
import {
  filtersOf,
  jobsHref,
  tabOf,
  type JobsParams,
} from "../../view/jobs-links.ts";
import { WorkspacePageView } from "../workspace-page.tsx";
import { HistoryView } from "./history-view.tsx";
import { LiveView } from "./live-view.tsx";

function JobsPage({ workspaceId }: { workspaceId: string }) {
  const params = useLocalSearchParams<JobsParams>();
  const tab = tabOf(params);
  const filters = filtersOf(params);
  return (
    <>
      <PageHeader title="Jobs" />
      <LinkTabs
        label="Jobs views"
        current={tab}
        tabs={[
          {
            key: "live",
            label: "Live",
            href: jobsHref(workspaceId, "live"),
            live: true,
          },
          {
            key: "history",
            label: "History",
            href: jobsHref(workspaceId, "history"),
          },
        ]}
      />
      {tab === "live" ? (
        <LiveView workspaceId={workspaceId} />
      ) : (
        <HistoryView workspaceId={workspaceId} filters={filters} />
      )}
    </>
  );
}

export function JobsScreen() {
  return (
    <WorkspacePageView page="jobs">
      {(context) => <JobsPage workspaceId={context.workspace.id} />}
    </WorkspacePageView>
  );
}
