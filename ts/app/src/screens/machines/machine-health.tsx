// The health of one machine: last contact, KVM, free disk, and the agent
// version. While the machine is offline, only the last contact is current:
// the other checks show the values of the last contact in grey, with a note.
// A machine without KVM shows that check as failed.

import { StyleSheet, Text } from "react-native";

import type { MachineView } from "../../api/schema.ts";
import {
  DetailRow,
  Panel,
  StatusLabel,
} from "../../components/feature-parts.tsx";
import { text } from "../../components/text.ts";
import { healthCopy } from "../../view/machines-copy.ts";
import { healthModel } from "../../view/machines-panels.ts";

export function HealthPanel({
  machine,
  now,
}: {
  machine: MachineView;
  now: number;
}) {
  const model = healthModel(machine, now);
  return (
    <Panel title={healthCopy.title}>
      <>
        {model.rows.map((row, index) => (
          <DetailRow key={row.key} label={row.label} first={index === 0}>
            <StatusLabel tone={row.tone} muted={row.old}>
              {row.value}
            </StatusLabel>
          </DetailRow>
        ))}
      </>
      {model.note ? (
        <Text style={[text.caption, styles.note]}>{model.note}</Text>
      ) : null}
    </Panel>
  );
}

const styles = StyleSheet.create({ note: { marginTop: -2 } });
