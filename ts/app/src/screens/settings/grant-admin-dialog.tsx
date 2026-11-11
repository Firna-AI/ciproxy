// The dialog before an Admin gives a person the Admin role. The Admin role
// alone adds machines and connects Boat, which run the jobs and their
// secrets of every organization of the workspace. So the dialog shows the
// trust notice of the API, and says that an organization whose runners other
// people should control uses a workspace of its own.

import { useState } from "react";
import { StyleSheet, Text, View } from "react-native";

import type { ApiResult } from "../../api/client.ts";
import type { Success } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { FeatureDialog } from "../../components/feature-parts.tsx";
import { Notice } from "../../components/notice.tsx";
import { text } from "../../components/text.ts";
import type { FailureCopy } from "../../view/claim.ts";
import {
  memberFailure,
  type MemberAction,
} from "../../view/settings-members.ts";

export function GrantAdminDialog({
  name,
  notice,
  action,
  grant,
  onClose,
}: {
  name: string;
  /** The trust notice of the API. */
  notice: string;
  /** A role change of a member, or the grant of a request. */
  action: Extract<MemberAction, "role" | "grant">;
  grant: () => Promise<ApiResult<Success>>;
  onClose: () => void;
}) {
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<FailureCopy | null>(null);
  const press = async () => {
    setBusy(true);
    setFailure(null);
    const result = await grant();
    setBusy(false);
    if (result.ok) onClose();
    else setFailure(memberFailure(result.failure, action));
  };
  return (
    <FeatureDialog
      title={`Give ${name} the Admin role?`}
      visible
      busy={busy}
      onClose={onClose}
      footer={
        <View style={styles.footer}>
          <ActionButton
            label="Cancel"
            tone="ghost"
            disabled={busy}
            onPress={onClose}
          />
          <ActionButton
            label="Give the Admin role"
            tone="primary"
            busy={busy}
            disabled={busy}
            onPress={() => void press()}
          />
        </View>
      }
    >
      <View style={styles.body}>
        {failure ? (
          <Notice tone="danger" title={failure.title}>
            {failure.body}
          </Notice>
        ) : null}
        <Notice
          tone="warning"
          title="An Admin controls the runners of every organization"
        >
          {notice}
        </Notice>
        <Text style={text.small}>
          If other people should control the runners of an organization, give it
          a workspace of its own.
        </Text>
      </View>
    </FeatureDialog>
  );
}

const styles = StyleSheet.create({
  body: { gap: 12 },
  footer: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
});
