// The open step of Setup: the machine step with the install command, and
// the first-job step. Every name, count, and result comes from the API. A
// machine that connected and gets its first job image says so, and a first
// job that waits for that machine says why. The first-job step also copies a
// prompt for a coding agent.

import type { ReactNode } from "react";
import { Link } from "expo-router";
import { StyleSheet, Text, View } from "react-native";

import { colors, radii, statusColors } from "@ciproxy/tokens";

import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { CodeBlock } from "../../components/code-block.tsx";
import { afterText } from "../../components/copy-state.ts";
import { AlertIcon, DoneIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import { WaitingLine } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import { workspaceHref } from "../../routing/pages.ts";
import { imageCopy, noticeCopy } from "../../view/machines-copy.ts";
import type { SetupModel } from "../../view/setup-steps.ts";
import { ImageLine } from "../machines/image-line.tsx";
import { AgentPromptCopy } from "./agent-prompt-copy.tsx";
import { InstallCommandView } from "./install-command.tsx";

const workflowFile = ".github/workflows/ci.yml";

const runsOnDiff = [
  "  jobs:",
  "    test:",
  "-     runs-on: ubuntu-latest",
  "+     runs-on: ciproxy",
];

function Outcome({
  ok,
  title,
  children,
  note,
}: {
  ok: boolean;
  title: string;
  children: string;
  /** A line under the body, such as the first job image of a machine. */
  note?: ReactNode;
}) {
  return (
    <View style={styles.outcome}>
      {ok ? (
        <DoneIcon size={18} color={statusColors.ok} />
      ) : (
        <AlertIcon size={18} color={colors.ink2} />
      )}
      <View style={styles.outcomeText}>
        <Text style={[text.body, text.strong]}>{title}</Text>
        <Text style={text.small}>{children}</Text>
        {note}
      </View>
    </View>
  );
}

export function MachineStep({
  model,
  workspaceId,
  workspaceName,
  onContinue,
}: {
  model: SetupModel;
  workspaceId: string;
  workspaceName: string;
  /** Set while the step stays open after a machine joined. */
  onContinue: (() => void) | null;
}) {
  return (
    <View style={styles.body}>
      <Text style={text.small}>
        Run this command on a Linux machine or a Mac. It installs the ciproxy
        agent and connects the machine to {workspaceName}.
      </Text>
      {model.cannotStartVms ? (
        <Notice
          tone="danger"
          title={`${model.cannotStartVms.host} cannot start VMs`}
        >
          {model.cannotStartVms.fix}
        </Notice>
      ) : null}
      <InstallCommandView workspaceId={workspaceId} />
      {model.connected && onContinue ? (
        <>
          <Outcome
            ok
            title={`${model.connected.name} connected`}
            note={model.connected.preparing ? <ImageLine /> : null}
          >
            {[model.connected.facts, model.connected.diskNote]
              .filter(Boolean)
              .join(". ")}
          </Outcome>
          <View style={styles.row}>
            <ActionButton
              label="Continue"
              tone="primary"
              onPress={onContinue}
            />
          </View>
        </>
      ) : (
        <WaitingLine>Waiting for the machine to connect…</WaitingLine>
      )}
    </View>
  );
}

export function JobStep({
  model,
  workspaceId,
}: {
  model: SetupModel;
  workspaceId: string;
}) {
  const { job } = model;
  if (job.kind === "finished") {
    const jobs = workspaceHref("jobs", workspaceId);
    return (
      <View style={styles.body}>
        <Outcome ok={job.passed} title={job.title}>
          {job.text}
        </Outcome>
        <View style={styles.row}>
          <Link href={jobs} asChild>
            <ActionLink label="Open Jobs" href={jobs} tone="primary" />
          </Link>
        </View>
      </View>
    );
  }
  return (
    <View style={styles.body}>
      <Text style={text.small}>
        Change runs-on in one workflow file, then push the change.
      </Text>
      <CodeBlock
        label={workflowFile}
        lines={runsOnDiff}
        copy={{ text: afterText(runsOnDiff), subject: "the workflow change" }}
      />
      {model.agentPrompt === null ? null : (
        <AgentPromptCopy prompt={model.agentPrompt} />
      )}
      <WaitingLine>
        {job.kind === "moving"
          ? job.text
          : "Waiting for a job with runs-on: ciproxy…"}
      </WaitingLine>
      {model.imageWait === null ? null : (
        <Notice tone="info" title={noticeCopy.preparingTitle(model.imageWait)}>
          {imageCopy.firstJob}
        </Notice>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  body: { gap: 12 },
  row: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
  outcome: {
    flexDirection: "row",
    alignItems: "flex-start",
    gap: 10,
    paddingVertical: 12,
    paddingHorizontal: 14,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.lg,
    backgroundColor: colors.surface,
  },
  outcomeText: { flex: 1, gap: 2 },
});
