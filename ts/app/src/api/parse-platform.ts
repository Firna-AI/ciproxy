// Readers for the platform of a label or a machine, and for whether a machine
// can start job VMs. A Linux machine needs KVM; a Mac needs
// Virtualization.framework, its entitlement, and a macOS that runs the image.

import { oneOf } from "./parse-core.ts";
import type { Platform, Virtualization } from "./schema.ts";

export const readPlatform = oneOf<Platform>({
  linux_x64: true,
  macos_arm64: true,
});

export const readVirtualization = oneOf<Virtualization>({
  available: true,
  kvm_unavailable: true,
  framework_unsupported: true,
  entitlement_missing: true,
  operating_system_too_old: true,
});
