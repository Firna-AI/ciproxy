// The form of the repository access of one organization: all private
// repositories, which includes the new ones, or the selected ones, with a
// check box for each private repository. Save stays off until the access
// changes. A public repository never gets a ciproxy runner, so the form
// names the count of them and offers no check box.

import { useState } from "react";
import { StyleSheet, Text, View } from "react-native";
import { RadioCard, RadioCardGroup } from "@firna/ui/radio";

import { colors, radii } from "@ciproxy/tokens";

import type { OrganizationView } from "../../api/schema.ts";
import type { RepositoryView } from "../../api/settings-schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { Notice } from "../../components/notice.tsx";
import { text } from "../../components/text.ts";
import type { SettingsStore } from "../../state/settings-store.ts";
import type { FailureCopy } from "../../view/claim.ts";
import { organizationFailure } from "../../view/settings-organizations.ts";
import {
  accessDraft,
  accessTexts,
  changed,
  selectionOf,
  toggled,
} from "../../view/settings-repositories.ts";
import { RepositoryCheck } from "./repository-check.tsx";

export function AccessForm({
  store,
  organization,
  repositories,
}: {
  store: SettingsStore;
  organization: OrganizationView;
  repositories: RepositoryView[];
}) {
  const [draft, setDraft] = useState(() =>
    accessDraft(organization, repositories),
  );
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<FailureCopy | null>(null);
  const texts = accessTexts(organization.login, draft, repositories);
  const dirty = changed(organization, draft, repositories);
  const save = async () => {
    setBusy(true);
    setFailure(null);
    const result = await store.saveRepositories(
      organization.id,
      selectionOf(draft, repositories),
    );
    setBusy(false);
    if (!result.ok)
      setFailure(organizationFailure(result.failure, "repositories"));
  };
  return (
    <View style={styles.form}>
      {failure ? (
        <Notice tone="danger" title={failure.title}>
          {failure.body}
        </Notice>
      ) : null}
      <RadioCardGroup label="Repository access">
        <RadioCard
          title="All private repositories"
          body={texts.allBody}
          checked={draft.mode === "all_private"}
          disabled={busy}
          onPress={() => setDraft({ ...draft, mode: "all_private" })}
        />
        <RadioCard
          title="Selected repositories"
          body={texts.selectedBody}
          checked={draft.mode === "selected"}
          disabled={busy}
          onPress={() => setDraft({ ...draft, mode: "selected" })}
        />
      </RadioCardGroup>
      {draft.mode === "selected" ? (
        <View style={styles.list}>
          {repositories
            .filter((repo) => repo.private)
            .map((repo, index) => (
              <RepositoryCheck
                key={repo.id}
                first={index === 0}
                name={repo.full_name}
                checked={draft.picked.has(repo.id)}
                disabled={busy}
                onToggle={() => setDraft(toggled(draft, repo.id))}
              />
            ))}
        </View>
      ) : null}
      {texts.publicNote ? (
        <Text style={text.caption}>{texts.publicNote}</Text>
      ) : null}
      <View style={styles.row}>
        <ActionButton
          label="Save"
          tone="primary"
          busy={busy}
          disabled={busy || !dirty}
          onPress={() => void save()}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  form: { gap: 16, maxWidth: 560 },
  list: {
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.lg,
    overflow: "hidden",
  },
  row: { flexDirection: "row" },
});
