// The prompt that the first-job step of Setup copies for a coding agent. It
// moves each job of a repository to the ciproxy label that fits the size of
// its current runner. The labels and their sizes come from the Setup
// response. A macOS label shows only while a Mac of the workspace can start
// job VMs, because its jobs run on the workspace's Macs only. A label that
// may run on no machine and not on Boat never starts a job, so it stays out.

import type { LabelInput, LabelView, MachineView } from "../api/schema.ts";
import { canStartVms, isMac, isMacLabel, platformName } from "./platform.ts";

function canRun(label: LabelInput, macReady: boolean): boolean {
  if (isMacLabel(label)) return macReady && label.allow_machines;
  return label.allow_machines || label.allow_boat;
}

/** Linux first, then macOS. Smaller sizes first in each platform. */
function bySize(a: LabelInput, b: LabelInput): number {
  return (
    Number(isMacLabel(a)) - Number(isMacLabel(b)) ||
    a.vcpus - b.vcpus ||
    a.memory_gb - b.memory_gb ||
    a.disk_gb - b.disk_gb ||
    a.name.localeCompare(b.name)
  );
}

function labelLine(label: LabelInput): string {
  const platform =
    platformName[isMacLabel(label) ? "macos_arm64" : "linux_x64"];
  return `- ${label.name}: ${platform}, ${label.vcpus} vCPU, ${label.memory_gb} GB memory, ${label.disk_gb} GB free disk`;
}

/**
 * The prompt for a coding agent, or null when no label of the workspace can
 * run a job.
 */
export function agentPrompt(
  labels: readonly LabelView[],
  machines: readonly MachineView[],
): string | null {
  const macReady = machines.some(
    (machine) => isMac(machine) && canStartVms(machine),
  );
  const usable = labels
    .map((view) => view.label)
    .filter((label) => canRun(label, macReady))
    .sort(bySize);
  if (usable.length === 0) return null;
  const kept = macReady
    ? "on Windows, or on Linux on Arm"
    : "on Windows, on macOS, or on Linux on Arm";
  return [
    "Move the GitHub Actions jobs of this repository to ciproxy runners. ciproxy runs the jobs of private repositories only. If this repository is public, stop and tell me.",
    "",
    "In each workflow file under .github/workflows/, find each job whose runs-on names a GitHub-hosted runner. Set its runs-on to the smallest of these labels, on the same platform, with at least the vCPU and memory of its current runner. Use the size that GitHub documents for that runner on a private repository.",
    "",
    ...usable.map(labelLine),
    "",
    "Change the runs-on values in a matrix and in the reusable workflows of this repository in the same way. Change no other line.",
    "",
    `Do not change a job when no label is large enough, or when it runs ${kept}. Ask me before you change a job that runs on a self-hosted runner or on another runner service, or a job whose runner size you do not know. In each question, suggest the label that you think fits, and say why. Wait for my answer before you change that job.`,
    "",
    "Commit and push the change as this repository usually does. Then list each job that you changed with its new label, and each job that you did not change with the reason.",
  ].join("\n");
}
