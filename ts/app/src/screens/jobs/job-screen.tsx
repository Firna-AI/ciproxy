// One job of a workspace, opened from History: where it ran and why, the
// times that GitHub gave, and its details. The page lives in the shell with
// Jobs active, and its breadcrumb leads back to History. A job that the
// history of the plan does not hold has its own state.

import { Link, useLocalSearchParams } from "expo-router";
import { StyleSheet, View } from "react-native";
import { SkeletonBar, SkeletonGroup } from "@firna/ui/skeleton";

import { failureStatus } from "../../api/client.ts";
import { ActionLink } from "../../components/actions.tsx";
import { DetailHeader } from "../../components/detail-header.tsx";
import { ExternalIcon, SearchIcon } from "../../components/icons.tsx";
import { ReadFailure, StateBlock } from "../../components/page-state.tsx";
import { Panel } from "../../components/panel.tsx";
import { StatusLabel } from "../../components/status-label.tsx";
import { useViewport } from "../../components/viewport.ts";
import { idParam } from "../../routing/pages.ts";
import { useRedirect } from "../../services/hooks.ts";
import { jobDetailModel } from "../../view/jobs-detail.ts";
import { jobsHref } from "../../view/jobs-links.ts";
import { WorkspacePageView } from "../workspace-page.tsx";
import { DetailsPanel, PlacePanel } from "./job-panels.tsx";
import { useJob } from "./jobs-hooks.ts";

function Skeleton() {
  return (
    <SkeletonGroup direction="column" gap={12}>
      <SkeletonBar width="60%" height={18} />
      <SkeletonBar width="85%" height={14} />
      <SkeletonBar width="50%" height={14} />
    </SkeletonGroup>
  );
}

function JobPage({
  workspaceId,
  jobId,
}: {
  workspaceId: string;
  jobId: string;
}) {
  const { snapshot, retry } = useJob(workspaceId, jobId);
  const wide = useViewport() === "wide";
  const history = jobsHref(workspaceId, "history");
  const crumbs = [{ label: "Jobs", href: history }];
  if (snapshot.status !== "ready") {
    const missing =
      snapshot.status === "failed" && failureStatus(snapshot.failure) === 404;
    return (
      <>
        <DetailHeader crumbs={crumbs} title="Job" />
        {snapshot.status === "loading" ? (
          <View style={[styles.panels, wide && styles.wide]}>
            <View style={wide ? styles.place : undefined}>
              <Panel title="Where it ran">
                <Skeleton />
              </Panel>
            </View>
            <View style={wide ? styles.details : undefined}>
              <Panel title="Details">
                <Skeleton />
              </Panel>
            </View>
          </View>
        ) : missing ? (
          <Panel>
            <StateBlock
              icon={SearchIcon}
              title="This job is not in your history"
              action={
                <Link href={history} asChild>
                  <ActionLink
                    label="Open History"
                    href={history}
                    tone="primary"
                  />
                </Link>
              }
            >
              It is older than the job history of your plan, or it belongs to
              another workspace.
            </StateBlock>
          </Panel>
        ) : (
          <Panel>
            <ReadFailure subject="the job" onRetry={retry} />
          </Panel>
        )}
      </>
    );
  }
  const model = jobDetailModel(snapshot.page.job);
  return (
    <>
      <DetailHeader
        crumbs={crumbs}
        title={model.title}
        status={
          <StatusLabel tone={model.look.tone}>{model.look.label}</StatusLabel>
        }
        subtitle={model.subtitle}
        actions={
          model.githubUrl ? (
            <ActionLink
              label="View on GitHub"
              href={model.githubUrl}
              icon={ExternalIcon}
              newTab
            />
          ) : null
        }
      />
      <View style={[styles.panels, wide && styles.wide]}>
        <View style={wide ? styles.place : undefined}>
          <PlacePanel model={model} />
        </View>
        <View style={wide ? styles.details : undefined}>
          <DetailsPanel model={model} />
        </View>
      </View>
    </>
  );
}

/** An address with no job opens History of the workspace. */
function ToHistory({ workspaceId }: { workspaceId: string }) {
  useRedirect(jobsHref(workspaceId, "history"));
  return null;
}

export function JobScreen() {
  const params = useLocalSearchParams<{ job?: string | string[] }>();
  const jobId = idParam(params.job);
  return (
    <WorkspacePageView page="jobs">
      {(context) =>
        jobId === null ? (
          <ToHistory workspaceId={context.workspace.id} />
        ) : (
          <JobPage workspaceId={context.workspace.id} jobId={jobId} />
        )
      }
    </WorkspacePageView>
  );
}

const styles = StyleSheet.create({
  panels: { gap: 16 },
  wide: { flexDirection: "row", alignItems: "flex-start" },
  /**
   * The 3:2 columns apply only in a row. In a column, a zero basis would let a
   * panel collapse to no height, so the next panel would cover it.
   */
  place: { flexGrow: 3, flexShrink: 1, flexBasis: 0, minWidth: 0 },
  details: { flexGrow: 2, flexShrink: 1, flexBasis: 0, minWidth: 0 },
});
