// The slots of one machine, each with its job or its warm runner, and the
// warm runner count. A busy slot names its job, with the repository and how
// long it has run, and the name opens the page of that job. A slot whose job
// the API could not match says only that it runs a job.

import { StyleSheet, Text, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import type { MachineView } from "../../api/schema.ts";
import { Panel, TextLink } from "../../components/feature-parts.tsx";
import { text } from "../../components/text.ts";
import type { WarmControl } from "../../view/machines-controls.ts";
import { detailCopy } from "../../view/machines-copy.ts";
import type { SlotJob, SlotsModel } from "../../view/machines-slots.ts";
import { slotCaption } from "../../view/machines.ts";
import { markRadius, SlotStrip, slotFill } from "./machine-figures.tsx";
import { WarmRunners } from "./machine-warm.tsx";

function JobText({ job, href }: { job: SlotJob; href: string }) {
  return (
    <>
      <View style={styles.job}>
        <TextLink href={href} label={job.name} />
        <Text style={text.caption}>{job.context}</Text>
      </View>
      {job.duration === null ? null : (
        <Text style={[text.small, styles.duration]}>{job.duration}</Text>
      )}
    </>
  );
}

function SlotList({
  model,
  jobHref,
}: {
  model: Extract<SlotsModel, { kind: "slots" }>;
  jobHref: (jobId: string) => string;
}) {
  return (
    <View role="list">
      {model.rows.map((row, index) => (
        <View
          key={row.key}
          role="listitem"
          style={[styles.slotRow, index > 0 && styles.divider]}
        >
          <View
            style={[styles.slot, { backgroundColor: slotFill[row.state] }]}
          />
          <Text style={[text.caption, styles.slotNumber]}>{row.label}</Text>
          {row.job === null ? (
            <Text
              style={[
                text.small,
                styles.text,
                row.state === "job" ? styles.ink : styles.muted,
              ]}
            >
              {row.text}
            </Text>
          ) : (
            <JobText job={row.job} href={jobHref(row.job.id)} />
          )}
        </View>
      ))}
    </View>
  );
}

export function SlotsPanel({
  machine,
  model,
  jobsHref,
  jobHref,
  warm,
}: {
  machine: MachineView;
  model: SlotsModel;
  jobsHref: string;
  /** The page of one dashboard job, by its canonical identity. */
  jobHref: (jobId: string) => string;
  warm: WarmControl;
}) {
  return (
    <Panel
      title={detailCopy.slots}
      action={<TextLink href={jobsHref} label={detailCopy.jobHistory} />}
    >
      {model.kind === "message" ? (
        <Text style={[text.small, styles.muted]}>{model.text}</Text>
      ) : (
        <>
          <SlotStrip states={model.states} label={slotCaption(machine)} />
          <SlotList model={model} jobHref={jobHref} />
          <Text style={text.caption}>{model.caption}</Text>
          {model.fits ? <Text style={text.caption}>{model.fits}</Text> : null}
        </>
      )}
      <WarmRunners control={warm} />
    </Panel>
  );
}

const styles = StyleSheet.create({
  slotRow: {
    flexDirection: "row",
    alignItems: "center",
    gap: 10,
    paddingVertical: 8,
  },
  divider: { borderTopWidth: 1, borderTopColor: colors.border },
  slot: { width: 14, height: 6, borderRadius: markRadius },
  slotNumber: { minWidth: 48 },
  text: { flex: 1, minWidth: 0 },
  job: {
    flex: 1,
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "baseline",
    columnGap: 8,
    minWidth: 0,
  },
  duration: { color: colors.ink, fontVariant: ["tabular-nums"] },
  ink: { color: colors.ink },
  muted: { color: colors.muted },
});
