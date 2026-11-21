// The steps of Setup as a list: a marker with the number, a check, or the
// dash of a skipped step, the title, and the open body or the line of a
// closed step. A finished or running first job keeps its body open. After a
// skip or a connection of Boat, focus moves to the first job.

import { useRef } from "react";
import { Link } from "expo-router";
import { StyleSheet, Text, View } from "react-native";

import { colors, fonts, radii } from "@ciproxy/tokens";

import { CheckIcon, MinusIcon } from "../../components/icons.tsx";
import { text } from "../../components/text.ts";
import { overflowHref } from "../../routing/pages.ts";
import type { SetupModel, SetupStep } from "../../view/setup-steps.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { BoatStep } from "./boat-step.tsx";
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
      ) : step.status === "skipped" ? (
        <MinusIcon size={14} strokeWidth={2.5} color={colors.muted} />
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

/** The line of a skipped Boat step, which leads to Settings under Overflow. */
function SkippedLine({ workspaceId }: { workspaceId: string }) {
  const href = overflowHref(workspaceId);
  return (
    <Text style={text.small}>
      Skipped for now. Connect Boat in{" "}
      <Link href={href} style={styles.link}>
        Settings under Overflow
      </Link>
      .
    </Text>
  );
}

function StepBody(props: {
  step: SetupStep;
  model: SetupModel;
  context: WorkspaceContext;
  onContinue: (() => void) | null;
  onBoatResolved: () => void;
}) {
  const { step, model, context } = props;
  const workspaceId = context.workspace.id;
  if (step.status === "active" && step.key === "machine") {
    return (
      <MachineStep
        model={model}
        workspaceId={workspaceId}
        workspaceName={context.workspace.name}
        onContinue={props.onContinue}
      />
    );
  }
  if (step.status === "active" && step.key === "boat") {
    return (
      <BoatStep
        workspaceId={workspaceId}
        login={context.session.login}
        onResolved={props.onBoatResolved}
      />
    );
  }
  if (step.status === "skipped")
    return <SkippedLine workspaceId={workspaceId} />;
  if (
    step.key === "job" &&
    (step.status === "active" || model.job.kind === "finished")
  ) {
    return <JobStep model={model} workspaceId={workspaceId} />;
  }
  return <Text style={text.small}>{step.summary}</Text>;
}

export function Steps(props: {
  model: SetupModel;
  context: WorkspaceContext;
  onContinue: (() => void) | null;
}) {
  const { model, context, onContinue } = props;
  const jobTitle = useRef<View>(null);
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
            <View
              ref={step.key === "job" ? jobTitle : undefined}
              tabIndex={step.key === "job" ? -1 : undefined}
            >
              <Text
                role="heading"
                aria-level={2}
                style={[
                  styles.title,
                  step.status === "todo" && styles.titleTodo,
                ]}
              >
                {step.title}
              </Text>
            </View>
            <StepBody
              step={step}
              model={model}
              context={context}
              onContinue={onContinue}
              onBoatResolved={() => jobTitle.current?.focus()}
            />
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
  link: { color: colors.ink, textDecorationLine: "underline" },
});
