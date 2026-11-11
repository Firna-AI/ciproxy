// The monthly cost of one machine, which the savings report uses. An Admin
// edits it in place: the field takes US dollars, and Save sends integer
// cents. A known zero stays apart from a cost that nobody entered. A wrong
// amount or a failed save shows under the field, and the field keeps its
// text, so nothing that the person typed is lost.

import { useState } from "react";
import { StyleSheet, Text, View } from "react-native";
import { Input } from "@firna/ui/input";

import type { MachineView } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { Panel } from "../../components/feature-parts.tsx";
import { text } from "../../components/text.ts";
import type { CostEdit } from "../../view/machines-controls.ts";
import { costCopy } from "../../view/machines-copy.ts";
import { parseDollars } from "../../view/machines-format.ts";
import { failureText } from "../../view/machines-outcome.ts";
import { costModel } from "../../view/machines-panels.ts";

function CostForm({
  initial,
  edit,
  onClose,
}: {
  initial: string;
  edit: CostEdit;
  onClose: () => void;
}) {
  const [value, setValue] = useState(initial);
  const [invalid, setInvalid] = useState(false);
  const submit = async () => {
    const cents = parseDollars(value);
    setInvalid(cents === null);
    if (cents === null) return;
    if (await edit.save(cents)) onClose();
  };
  const error = invalid
    ? costCopy.invalid
    : edit.failure
      ? costCopy.failed(failureText(edit.failure))
      : null;
  return (
    <View style={styles.form}>
      <Input
        label={costCopy.field}
        value={value}
        onChangeText={(next) => {
          setValue(next);
          setInvalid(false);
        }}
        onSubmitEditing={() => void submit()}
        inputMode="decimal"
        error={error}
        editable={!edit.busy}
      />
      <View style={styles.actions}>
        <ActionButton
          label={costCopy.save}
          tone="primary"
          size="sm"
          busy={edit.busy}
          onPress={() => void submit()}
        />
        <ActionButton
          label={costCopy.cancel}
          tone="ghost"
          size="sm"
          disabled={edit.busy}
          onPress={onClose}
        />
      </View>
    </View>
  );
}

export function CostPanel({
  machine,
  edit,
}: {
  machine: MachineView;
  /** Null for a person who cannot change the cost. */
  edit: CostEdit | null;
}) {
  const [open, setOpen] = useState(false);
  const model = costModel(machine);
  const close = () => {
    edit?.reset();
    setOpen(false);
  };
  return (
    <Panel title={costCopy.title}>
      {open && edit !== null ? (
        <CostForm initial={model.input} edit={edit} onClose={close} />
      ) : (
        <View style={styles.spread}>
          <View style={styles.value}>
            <Text style={[text.body, text.strong]}>{model.value}</Text>
            <Text style={text.caption}>{model.caption}</Text>
          </View>
          {edit === null ? null : (
            <ActionButton
              label={costCopy.edit}
              size="sm"
              onPress={() => {
                edit.reset();
                setOpen(true);
              }}
            />
          )}
        </View>
      )}
    </Panel>
  );
}

const styles = StyleSheet.create({
  form: { gap: 10 },
  actions: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
  spread: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
  },
  value: { flexGrow: 1, flexShrink: 1, flexBasis: 180, gap: 2 },
});
