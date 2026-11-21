// The dialogs of the Organizations tab. A new workspace takes the name of
// its first organization, and an Admin can rename it; the GitHub
// organizations keep their names. A take-out removes the GitHub App from the
// organization: no new job of it starts on ciproxy, its running jobs finish,
// its jobs stay in the job history, and it can join a workspace again with a
// new install. Turn on asks first, because the organization then uses the
// machines, the labels, the settings, and the Boat account of the workspace.

import { useState, type ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";
import { Input } from "@firna/ui/input";

import type { ApiResult } from "../../api/client.ts";
import type { OrganizationView, Success } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { FeatureDialog } from "../../components/feature-parts.tsx";
import { Notice } from "../../components/notice.tsx";
import { text } from "../../components/text.ts";
import type { FailureCopy } from "../../view/claim.ts";
import {
  addedByText,
  organizationFailure,
  type OrganizationAction,
} from "../../view/settings-organizations.ts";

/** A dialog whose one change closes it on success, or shows its failure. */
function ChangeDialog({
  title,
  action,
  label,
  tone,
  disabled = false,
  run,
  onClose,
  children,
}: {
  title: string;
  action: OrganizationAction;
  label: string;
  tone: "primary" | "danger";
  disabled?: boolean;
  run: () => Promise<ApiResult<Success>>;
  onClose: () => void;
  children: ReactNode;
}) {
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<FailureCopy | null>(null);
  const press = async () => {
    setBusy(true);
    setFailure(null);
    const result = await run();
    setBusy(false);
    if (result.ok) onClose();
    else setFailure(organizationFailure(result.failure, action));
  };
  return (
    <FeatureDialog
      title={title}
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
            label={label}
            tone={tone}
            busy={busy}
            disabled={busy || disabled}
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
        {children}
      </View>
    </FeatureDialog>
  );
}

export function RenameDialog({
  name,
  rename,
  onClose,
}: {
  name: string;
  rename: (name: string) => Promise<ApiResult<Success>>;
  onClose: () => void;
}) {
  const [draft, setDraft] = useState(name);
  const next = draft.trim();
  return (
    <ChangeDialog
      title="Rename the workspace"
      action="rename"
      label="Save"
      tone="primary"
      disabled={next === "" || next === name}
      run={() => rename(next)}
      onClose={onClose}
    >
      <Input
        label="Name"
        value={draft}
        onChangeText={setDraft}
        hint="The GitHub organizations keep their names."
      />
    </ChangeDialog>
  );
}

export function TakeOutDialog({
  login,
  takeOut,
  onClose,
}: {
  login: string;
  takeOut: () => Promise<ApiResult<Success>>;
  onClose: () => void;
}) {
  return (
    <ChangeDialog
      title={`Take ${login} out of this workspace?`}
      action="take-out"
      label="Take out"
      tone="danger"
      run={takeOut}
      onClose={onClose}
    >
      <Text style={text.small}>
        No new job of {login} starts on ciproxy, and its running jobs finish.
        ciproxy removes the GitHub App from {login}.
      </Text>
      <Text style={text.small}>
        Its jobs stay in the job history, and {login} can join a workspace again
        later with a new install.
      </Text>
    </ChangeDialog>
  );
}

/** A current Admin turns on an organization that a former Admin added. */
export function TurnOnDialog({
  organization,
  turnOn,
  onClose,
}: {
  organization: OrganizationView;
  turnOn: () => Promise<ApiResult<Success>>;
  onClose: () => void;
}) {
  const { login } = organization;
  return (
    <ChangeDialog
      title={`Turn on ${login}?`}
      action="turn-on"
      label="Turn on"
      tone="primary"
      run={turnOn}
      onClose={onClose}
    >
      <Text style={text.small}>
        The jobs of {login} then run on the machines of this workspace, with its
        labels, settings, and Boat account.
      </Text>
      <Text style={text.small}>{addedByText(organization)}</Text>
    </ChangeDialog>
  );
}

const styles = StyleSheet.create({
  body: { gap: 12 },
  footer: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
});
