// The read states that every tab of Settings shares: loading, a failed
// first read with Try again, and a failed refresh, which keeps the last good
// view and says so. Each tab supplies its own loading layout.

import type { ReactNode } from "react";

import { ActionButton } from "../../components/actions.tsx";
import { Panel } from "../../components/feature-parts.tsx";
import { Notice } from "../../components/notice.tsx";
import { ReadFailure } from "../../components/page-state.tsx";
import type { ReadSnapshot } from "../../state/settings-store.ts";

export function RefreshFailure({
  subject,
  onRetry,
}: {
  subject: string;
  onRetry: () => void;
}) {
  return (
    <Notice
      tone="warning"
      title={`Unable to refresh ${subject}`}
      actions={<ActionButton label="Try again" size="sm" onPress={onRetry} />}
    >
      The page shows the last {subject} that it read.
    </Notice>
  );
}

export function SettingsRead<T>({
  snapshot,
  subject,
  onRetry,
  loading,
  children,
}: {
  snapshot: ReadSnapshot<T>;
  /** What failed to load, such as "settings" or "members". */
  subject: string;
  onRetry: () => void;
  loading: ReactNode;
  children: (value: T) => ReactNode;
}) {
  if (snapshot.status === "loading") return <>{loading}</>;
  if (snapshot.status === "failed") {
    return (
      <Panel>
        <ReadFailure subject={subject} onRetry={onRetry} />
      </Panel>
    );
  }
  return (
    <>
      {snapshot.refreshFailure ? (
        <RefreshFailure subject={subject} onRetry={onRetry} />
      ) : null}
      {children(snapshot.value)}
    </>
  );
}
