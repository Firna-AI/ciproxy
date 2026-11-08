// The Live view of Jobs: the figures, the notices of why jobs wait, and the
// waiting and running jobs. It reads again every few seconds while the tab
// shows. A failed refresh keeps the last queue and says so.

import { Text } from "react-native";

import { ActionButton } from "../../components/actions.tsx";
import { FigureRow } from "../../components/figures.tsx";
import { BoltIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import { ReadFailure, StateBlock } from "../../components/page-state.tsx";
import { Panel } from "../../components/panel.tsx";
import { text } from "../../components/text.ts";
import { workspaceHref } from "../../routing/pages.ts";
import { liveModel } from "../../view/jobs-live.ts";
import { liveNotices } from "../../view/jobs-notices.ts";
import { LiveList, LoadingList } from "./job-list.tsx";
import { useLive } from "./jobs-hooks.ts";
import { LiveNotices } from "./live-notices.tsx";

export function LiveView({ workspaceId }: { workspaceId: string }) {
  const { snapshot, retry } = useLive(workspaceId);
  if (snapshot.status === "loading") {
    return (
      <Panel title="Running" flush>
        <LoadingList kind="running" />
      </Panel>
    );
  }
  if (snapshot.status === "failed") {
    return (
      <Panel>
        <ReadFailure subject="jobs" onRetry={retry} />
      </Panel>
    );
  }
  const model = liveModel(snapshot.page, snapshot.readAt);
  const notices = liveNotices(
    snapshot.page,
    workspaceHref("settings", workspaceId),
  );
  return (
    <>
      {snapshot.refreshFailure ? (
        <Notice
          tone="warning"
          title="Unable to refresh the jobs"
          actions={<ActionButton label="Try again" size="sm" onPress={retry} />}
        >
          The page shows the last queue that it read, and tries again by itself.
        </Notice>
      ) : null}
      <FigureRow figures={model.figures} />
      <LiveNotices notices={notices} />
      {model.quiet ? (
        <Panel>
          <StateBlock icon={BoltIcon} title={model.quiet.title}>
            {model.quiet.warm ? (
              <>
                The next job with{" "}
                <Text style={text.mono}>runs-on: ciproxy</Text> starts at once
                on one of your machines.
              </>
            ) : (
              <>
                Jobs with <Text style={text.mono}>runs-on: ciproxy</Text> show
                here when they wait or run.
              </>
            )}
          </StateBlock>
        </Panel>
      ) : (
        <>
          {model.waiting.length > 0 ? (
            <Panel title={`Waiting · ${model.waiting.length}`} flush>
              <LiveList kind="waiting" rows={model.waiting} />
            </Panel>
          ) : null}
          <Panel title={`Running · ${model.running.length}`} flush>
            <LiveList kind="running" rows={model.running} />
          </Panel>
        </>
      )}
    </>
  );
}
