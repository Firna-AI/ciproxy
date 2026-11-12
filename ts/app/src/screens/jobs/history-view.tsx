// The History view of Jobs: the filters, then the finished jobs that match
// them, newest first, as the API orders them. The filters stay in every
// state, also when no job matches. The footer counts the jobs of the list.

import { Link } from "expo-router";
import { StyleSheet, Text, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import { ActionLink } from "../../components/actions.tsx";
import { JobsIcon, SearchIcon } from "../../components/icons.tsx";
import { ReadFailure, StateBlock } from "../../components/page-state.tsx";
import { Panel } from "../../components/panel.tsx";
import { text } from "../../components/text.ts";
import {
  finishedRow,
  historyEmpty,
  historyFooter,
} from "../../view/jobs-history.ts";
import {
  defaultFilters,
  jobsHref,
  type HistoryFilters,
} from "../../view/jobs-links.ts";
import { HistoryFiltersView } from "./history-filters.tsx";
import { FinishedList, LoadingList } from "./job-list.tsx";
import { useHistory } from "./jobs-hooks.ts";

function EmptyList({
  workspaceId,
  filters,
}: {
  workspaceId: string;
  filters: HistoryFilters;
}) {
  const empty = historyEmpty(filters);
  const clear = jobsHref(workspaceId, "history", defaultFilters);
  return (
    <StateBlock
      icon={empty.clear ? SearchIcon : JobsIcon}
      title={empty.title}
      action={
        empty.clear ? (
          <Link href={clear} asChild>
            <ActionLink label="Clear filters" href={clear} tone="primary" />
          </Link>
        ) : null
      }
    >
      {empty.body}
    </StateBlock>
  );
}

export function HistoryView({
  workspaceId,
  filters,
}: {
  workspaceId: string;
  filters: HistoryFilters;
}) {
  const { snapshot, retry } = useHistory(workspaceId, filters);
  const shown =
    snapshot.status === "ready"
      ? snapshot.page
      : snapshot.status === "loading"
        ? (snapshot.shown?.page ?? null)
        : null;
  return (
    <>
      <HistoryFiltersView
        workspaceId={workspaceId}
        filters={filters}
        historyDays={shown?.history_days ?? null}
      />
      {snapshot.status === "failed" ? (
        <Panel>
          <ReadFailure subject="finished jobs" onRetry={retry} />
        </Panel>
      ) : shown === null ? (
        <Panel flush>
          <LoadingList kind="finished" />
        </Panel>
      ) : shown.jobs.length === 0 ? (
        <Panel>
          <EmptyList workspaceId={workspaceId} filters={filters} />
        </Panel>
      ) : (
        <Panel flush>
          <FinishedList
            rows={shown.jobs.map((job) => finishedRow(job, workspaceId))}
          />
          <View style={styles.footer}>
            <Text style={text.caption}>
              {historyFooter(shown.jobs.length, filters, shown.history_days)}
            </Text>
          </View>
        </Panel>
      )}
    </>
  );
}

const styles = StyleSheet.create({
  footer: {
    paddingVertical: 10,
    paddingHorizontal: 16,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
});
