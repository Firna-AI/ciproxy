// The platform of a label or a machine, as the dashboard names it. A Linux
// machine is an x64 computer that runs Linux, and a Mac is an Apple Silicon
// computer that runs macOS. A label keeps its platform, and its jobs run only
// on machines of that platform. The jobs of a macOS label never run on an
// overflow provider.

import type {
  LabelInput,
  MachineView,
  Platform,
  Virtualization,
} from "../api/schema.ts";

export const platformName: Readonly<Record<Platform, string>> = {
  linux_x64: "Linux",
  macos_arm64: "macOS",
};

/** The platform of a label: an absent value means Linux, as the API says. */
export function labelPlatform(label: Pick<LabelInput, "platform">): Platform {
  return label.platform ?? "linux_x64";
}

export function isMacLabel(label: Pick<LabelInput, "platform">): boolean {
  return labelPlatform(label) === "macos_arm64";
}

export function isMac(machine: Pick<MachineView, "platform">): boolean {
  return machine.platform === "macos_arm64";
}

/**
 * The default label of the platform of a machine, which the API counts in
 * the jobs that the machine runs at a time.
 */
export function defaultLabelName(machine: Pick<MachineView, "platform">) {
  return isMac(machine) ? "ciproxy-macos" : "ciproxy";
}

/** The machine can start job VMs. */
export function canStartVms(
  machine: Pick<MachineView, "virtualization">,
): boolean {
  return machine.virtualization === "available";
}

/** Why a machine cannot start job VMs, and the fix. */
const virtualizationFix: Readonly<
  Record<Exclude<Virtualization, "available">, string>
> = {
  kvm_unavailable:
    "ciproxy runs each job in its own VM, which needs KVM. Turn on virtualization for this machine, then run the command again.",
  framework_unsupported:
    "ciproxy runs each job in a macOS VM, which needs a Mac with Apple Silicon. Use a Mac with an M1 chip or later.",
  entitlement_missing:
    "The agent cannot use the virtualization of macOS. Run the install command again to repair the agent.",
  operating_system_too_old:
    "Its macOS is older than the macOS of the job image. Update macOS, then run the command again.",
};

/** The fix of a machine that cannot start job VMs, or null when it can. */
export function virtualizationProblem(virtualization: Virtualization) {
  return virtualization === "available"
    ? null
    : virtualizationFix[virtualization];
}
