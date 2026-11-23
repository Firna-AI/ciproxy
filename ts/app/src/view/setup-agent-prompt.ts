// The prompt that the first-job step of Setup copies for a coding agent. It
// moves each job of a repository to the ciproxy label that fits the platform
// and the size of its current runner. The labels, their sizes, and their
// placement come from the Setup response. A label stays out of the prompt
// when no destination gives its full size: its placement is `waits`, or only
// the largest Boat size runs it, which is smaller than the label. A label
// whose placement is unknown, such as while a machine gets its first job
// image, stays in. The matrix example names no real runner or label, so that
// it cannot pair a runner with a label of another platform or size.

import type {
  LabelFit,
  LabelInput,
  LabelView,
  Platform,
} from "../api/schema.ts";
import { labelPlatform } from "./platform.ts";
import { compareLabels } from "./settings-labels.ts";

/** The operating system and the CPU architecture of a label's VM. */
const platformText: Readonly<Record<Platform, string>> = {
  linux_x64: "Linux on x64",
  macos_arm64: "macOS on Apple silicon (arm64)",
};

/** A destination runs the jobs of the label at the label's own size. */
function fullSize(fit: LabelFit): boolean {
  if (fit.placement === "waits") return false;
  return !(
    fit.placement === "boat_only" && fit.boat.state === "largest_fallback"
  );
}

function labelLine(label: LabelInput): string {
  const platform = platformText[labelPlatform(label)];
  return `- ${label.name}: ${platform}, ${label.vcpus} vCPU, ${label.memory_gb} GB memory, ${label.disk_gb} GB free disk`;
}

/**
 * The prompt for a coding agent, or null when no label of the workspace has
 * a destination that fits it.
 */
export function agentPrompt(labels: readonly LabelView[]): string | null {
  const usable = labels
    .filter((view) => fullSize(view.fit))
    .map((view) => view.label)
    .sort(compareLabels);
  if (usable.length === 0) return null;
  return [
    "Move the GitHub Actions jobs of this repository to ciproxy runners. ciproxy runs the jobs of private repositories only. If this repository is public, stop and tell me.",
    "",
    "In each workflow file under .github/workflows/, find each job whose runs-on names a GitHub-hosted runner. Set its runs-on to the smallest of these labels with the same operating system and CPU architecture as its current runner, and with at least its vCPU and memory. Use the size and the architecture that GitHub documents for that runner on a private repository.",
    "",
    ...usable.map(labelLine),
    "",
    "Change only the runner of each job. When the runs-on value comes from a matrix, an input, or a variable that other lines also read, keep that value as it is. Map it to the label in the runs-on line only, in this shape: runs-on: ${{ matrix.os == '<runner>' && '<label>' || matrix.os }}. Put in each runner and the label that you chose for it under the rules above. Change no other line.",
    "",
    "Do not change a job when no label has the operating system and the CPU architecture of its runner, such as a job on Windows, or when no label is large enough. Ask me before you change a job that runs on a self-hosted runner or on another runner service, or a job whose runner size or architecture you do not know. In each question, suggest the label that you think fits, and say why. Wait for my answer before you change that job.",
    "",
    "Commit and push the change as this repository usually does. Then list each job that you changed with its new label, and each job that you did not change with the reason.",
  ].join("\n");
}
