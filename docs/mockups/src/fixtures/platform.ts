// The platform of an own machine and of a label. A Linux machine is an x64
// computer that runs Linux, and a Mac is an Apple Silicon computer that runs
// macOS. A label keeps its platform, as it keeps its name, and its jobs run
// only on machines of that platform. Jobs of a macOS label never run on Boat,
// because a cloud Mac bills 24 hours at least. Apple allows at most 2 macOS
// VMs at a time on one Mac, so a Mac has at most 2 slots.

export type Platform = "linux" | "macos";

/** The platform as the product names it. */
export const platformName: Readonly<Record<Platform, string>> = {
  linux: "Linux",
  macos: "macOS",
};

/** The most macOS VMs that Apple allows at a time on one Mac. */
export const macosVmLimit = 2;

/**
 * The oldest macOS that a Mac runs: the macOS of the job image, because a
 * macOS VM cannot run a newer macOS than its host.
 */
export const minimumMacos = "macOS 26.7.3";
