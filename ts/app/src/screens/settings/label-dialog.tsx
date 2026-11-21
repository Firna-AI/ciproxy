// The dialog that adds a label or edits one. While the person types, it
// asks the API how many jobs of the size fit on each machine. A save goes to
// GitHub first for a new name, and the dialog shows the refusal of GitHub
// under the name. An edit keeps the name, and every label but `ciproxy`
// offers Delete label, which a rename uses with a new label.

import { useEffect, useState } from "react";
import { StyleSheet, View } from "react-native";

import type { LabelView } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { FeatureDialog } from "../../components/feature-parts.tsx";
import { Notice } from "../../components/notice.tsx";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { createLabelPreview, fitKey } from "../../state/settings-preview.ts";
import type { SettingsStore } from "../../state/settings-store.ts";
import type { FailureCopy } from "../../view/claim.ts";
import {
  checkDraft,
  editDraft,
  labelFailure,
  newDraft,
  type LabelDraft,
  type SizeField,
} from "../../view/settings-label-form.ts";
import { canDelete } from "../../view/settings-labels.ts";
import { LabelFitLine } from "./label-fit.tsx";
import { LabelForm } from "./label-form.tsx";

export type LabelTarget = { kind: "add" } | { kind: "edit"; label: LabelView };

export function LabelDialog({
  store,
  target,
  labels,
  onClose,
}: {
  store: SettingsStore;
  target: LabelTarget;
  labels: readonly LabelView[];
  onClose: () => void;
}) {
  const { timers } = useServices().env;
  const saved = target.kind === "edit" ? target.label : null;
  const [draft, setDraft] = useState<LabelDraft>(() =>
    saved === null ? newDraft(labels) : editDraft(saved.label),
  );
  const [showErrors, setShowErrors] = useState(false);
  const [nameError, setNameError] = useState<string | null>(null);
  const [failure, setFailure] = useState<FailureCopy | null>(null);
  const [busy, setBusy] = useState<"save" | "delete" | null>(null);
  // One preview for each open dialog, with the fit of the saved label first.
  const [preview] = useState(() =>
    createLabelPreview({
      preview: store.previewLabel,
      timers,
      initial: saved === null ? null : { input: saved.label, fit: saved.fit },
    }),
  );
  const previewState = useValue(preview.state);
  useEffect(() => () => preview.stop(), [preview]);

  const check = checkDraft(draft);
  const key = check.ok ? fitKey(check.input) : null;
  // Each change of the parts in `key` enters the preview store, which ends
  // the earlier work and decides between the shown fit and a new ask.
  useEffect(() => {
    preview.request(check.ok ? check.input : null);
  }, [key, preview]);

  const sizeErrors: Partial<Record<SizeField, string>> = {};
  if (!check.ok) {
    for (const field of ["vcpus", "memory", "disk"] as const) {
      const error = check.errors[field];
      if (error !== undefined && (showErrors || draft[field] !== "")) {
        sizeErrors[field] = error;
      }
    }
  }

  const change = (next: LabelDraft) => {
    if (next.name !== draft.name) setNameError(null);
    setDraft(next);
  };

  const save = async () => {
    if (!check.ok) {
      setShowErrors(true);
      return;
    }
    setBusy("save");
    setFailure(null);
    const result = await store.saveLabel(saved?.id ?? null, check.input);
    setBusy(null);
    if (result.ok) return onClose();
    const refused = labelFailure(result.failure, "save");
    if (refused.kind === "name") setNameError(refused.message);
    else setFailure(refused.copy);
  };

  const remove = async (labelId: string) => {
    setBusy("delete");
    setFailure(null);
    const result = await store.deleteLabel(labelId);
    setBusy(null);
    if (result.ok) return onClose();
    const refused = labelFailure(result.failure, "delete");
    setFailure(
      refused.kind === "form"
        ? refused.copy
        : { title: "Unable to delete the label", body: refused.message },
    );
  };

  const title = saved === null ? "Add a label" : `Edit ${saved.label.name}`;
  const footer = (
    <View style={styles.footer}>
      {saved !== null && canDelete(saved.label) ? (
        <ActionButton
          label="Delete label"
          tone="danger"
          busy={busy === "delete"}
          disabled={busy !== null}
          onPress={() => void remove(saved.id)}
        />
      ) : (
        <View />
      )}
      <View style={styles.end}>
        <ActionButton
          label="Cancel"
          tone="ghost"
          disabled={busy !== null}
          onPress={onClose}
        />
        <ActionButton
          label={saved === null ? "Add label" : "Save label"}
          tone="primary"
          busy={busy === "save"}
          disabled={busy !== null || nameError !== null || draft.name === ""}
          onPress={() => void save()}
        />
      </View>
    </View>
  );
  return (
    <FeatureDialog
      title={title}
      visible
      busy={busy !== null}
      onClose={onClose}
      footer={footer}
    >
      <View style={styles.body}>
        {failure ? (
          <Notice tone="danger" title={failure.title}>
            {failure.body}
          </Notice>
        ) : null}
        <LabelForm
          draft={draft}
          edit={saved !== null}
          nameError={nameError}
          sizeErrors={sizeErrors}
          fit={<LabelFitLine check={check} preview={previewState} />}
          onChange={change}
        />
      </View>
    </FeatureDialog>
  );
}

const styles = StyleSheet.create({
  body: { gap: 16 },
  footer: {
    flex: 1,
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 8,
  },
  end: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
});
