// The steps of Setup as a list: a marker with the number or a check, the
// title, and the open body or the line of a closed step. A finished first
// job keeps its body open, with the outcome and Open Jobs.

import { StyleSheet, Text, View } from "react-native";

import { colors, fonts, radii } from "@ciproxy/tokens";

import { CheckIcon } from "../../components/icons.tsx";
import { text } from "../../components/text.ts";
import type { SetupModel, SetupStep } from "../../view/setup-steps.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { JobStep, MachineStep } from "./step-bodies.tsx";

function Marker({ step, index }: { step: SetupStep; index: number }) {
  return (
    <View
      style={[
        styles.marker,
        step.status === "active" && styles.markerActive,
        step.status === "done" && styles.markerDone,
      ]}
    >
      {step.status === "done" ? (
        <CheckIcon size={14} strokeWidth={2.5} color={colors.ink} />
      ) : (
        <Text
          style={[
            styles.markerText,
            step.status === "active" && styles.markerTextActive,
          ]}
        >
          {index + 1}
        </Text>
      )}
    </View>
  );
}

export function Steps(props: {
  model: SetupModel;
  context: WorkspaceContext;
  onContinue: (() => void) | null;
}) {
  const { model, context, onContinue } = props;
  return (
    <View style={styles.steps} role="list">
      {model.steps.map((step, index) => (
        <View
          key={step.key}
          role="listitem"
          style={[
            styles.step,
            step.status === "active" && styles.stepActive,
            step.status === "todo" && styles.stepTodo,
          ]}
        >
          <Marker step={step} index={index} />
          <View style={styles.content}>
            <Text
              role="heading"
              aria-level={2}
              style={[styles.title, step.status === "todo" && styles.titleTodo]}
            >
              {step.title}
            </Text>
            {step.status === "active" && step.key === "machine" ? (
              <MachineStep
                model={model}
                workspaceId={context.workspace.id}
                workspaceName={context.workspace.name}
                onContinue={onContinue}
              />
            ) : step.key === "job" &&
              (step.status === "active" || model.job.kind === "finished") ? (
              <JobStep model={model} workspaceId={context.workspace.id} />
            ) : (
              <Text style={text.small}>{step.summary}</Text>
            )}
          </View>
        </View>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  steps: { gap: 12, maxWidth: 720 },
  step: {
    flexDirection: "row",
    gap: 14,
    padding: 16,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.xl,
    backgroundColor: colors.surface,
  },
  stepActive: { borderColor: colors.border2 },
  stepTodo: { backgroundColor: "transparent" },
  marker: {
    width: 24,
    height: 24,
    alignItems: "center",
    justifyContent: "center",
    borderWidth: 1,
    borderColor: colors.border2,
    borderRadius: radii.pill,
  },
  markerActive: { borderColor: colors.ink, backgroundColor: colors.ink },
  markerDone: { backgroundColor: colors.primarySoft },
  markerText: {
    fontFamily: fonts.mono,
    fontSize: 12,
    fontWeight: "500",
    color: colors.muted,
  },
  markerTextActive: { color: colors.onSolid },
  content: { flex: 1, gap: 6, minWidth: 0, paddingTop: 2 },
  title: {
    fontFamily: fonts.sans,
    fontSize: 14,
    fontWeight: "600",
    lineHeight: 20,
    color: colors.ink,
  },
  titleTodo: { color: colors.ink2 },
});
