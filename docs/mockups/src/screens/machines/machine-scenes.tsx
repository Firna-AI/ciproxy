// The states of the machine page, with one scene record each: the machine
// that the state shows, the Clear cache confirmation while it is open over
// the page, and the Clear cache confirmation that the page opens. Clear cache
// of the cache panel opens the confirmation of the state, and its Cancel
// returns to the page. A state with no confirmation of its own shows Clear
// cache as a plain button. Each page of hetzner-ax42-1 with unknown figures
// opens its confirmation over the same machine and figures. That
// confirmation names no size of the cache entries, also when the machine
// measured its machine cache, and there Clear cache acts in place.

import type { ReactNode } from "react";

import { paths, type CataloguePath } from "../../components/paths.ts";
import {
  macMiniMachine,
  macMiniOldMacos,
  macStudioMachine,
} from "../../fixtures/macs.ts";
import { machineScenes, type Machine } from "../../fixtures/machines.ts";
import { preparingAx42 } from "../../fixtures/new-machine.ts";
import {
  ClearCacheDialog,
  type ClearCacheConfirmation,
} from "./machine-cache.tsx";

export const machineStates = [
  "healthy",
  "draining",
  "offline",
  "no-slots",
  "disk-limit",
  "clear-cache",
  "clear-requested",
  "saved-offline",
  "no-room",
  "preparing",
  "agent-behind",
  "cache-unknown",
  "docker-unknown",
  "both-unknown",
  "clear-cache-unknown",
  "clear-docker-unknown",
  "clear-both-unknown",
  "mac",
  "mac-one-slot",
  "mac-no-virtualization",
] as const;
export type MachineState = (typeof machineStates)[number];

export type MachineDetailScene = {
  /** The machine that the page shows. */
  machine: Machine;
  /** The Clear cache confirmation, while it is open over the page. */
  overlay?: ReactNode;
  /** The Clear cache confirmation of the page, when its screen exists. */
  clearCache?: ClearCacheConfirmation;
};

/** A scene whose Clear cache opens a confirmation. */
type ClearableScene = MachineDetailScene & {
  clearCache: ClearCacheConfirmation;
};

/**
 * hetzner-ax42-1 on the busy afternoon of 18 November. The confirmation
 * names the size of the cache entries, and after the clear the page says
 * that the machine clears its cache.
 */
const steady: ClearableScene = {
  machine: machineScenes.steady[0],
  clearCache: {
    screen: paths.machineClearCache,
    cancel: paths.machine,
    cleared: paths.machineClearRequested,
    namesSize: true,
  },
};

/**
 * hetzner-ax42-1 in another state whose Clear cache opens the confirmation
 * of the busy afternoon, as the draining, saved-offline, and agent-behind
 * pages do.
 */
function steadyClear(machine: Machine): MachineDetailScene {
  return { machine, clearCache: steady.clearCache };
}

/**
 * A page of hetzner-ax42-1 with unknown figures. Its confirmation opens over
 * the same machine and names no size, and Cancel returns to the page. The
 * catalogue has no page after this clear, so Clear cache of the confirmation
 * acts in place.
 */
function unknownFigures(
  machine: Machine,
  page: CataloguePath,
  screen: CataloguePath,
): ClearableScene {
  return {
    machine,
    clearCache: { screen, cancel: page, namesSize: false },
  };
}

const cacheUnknown = unknownFigures(
  machineScenes.cacheUnknown[0],
  paths.machineCacheUnknown,
  paths.machineClearCacheUnknown,
);

const dockerUnknown = unknownFigures(
  machineScenes.dockerUnknown[0],
  paths.machineDockerUnknown,
  paths.machineClearDockerUnknown,
);

const bothUnknown = unknownFigures(
  machineScenes.figuresUnknown[0],
  paths.machineFiguresUnknown,
  paths.machineClearFiguresUnknown,
);

/** The scene with its Clear cache confirmation open over the page. */
function confirming(scene: ClearableScene): MachineDetailScene {
  return {
    ...scene,
    overlay: (
      <ClearCacheDialog
        machine={scene.machine}
        confirmation={scene.clearCache}
      />
    ),
  };
}

export function machineScene(state: MachineState): MachineDetailScene {
  switch (state) {
    case "healthy":
    case "clear-requested":
      return steady;
    case "clear-cache":
      return confirming(steady);
    case "draining":
      return steadyClear(machineScenes.draining[0]);
    case "saved-offline":
      return steadyClear(machineScenes.drainSaved[0]);
    case "agent-behind":
      return steadyClear(machineScenes.agentBehind[0]);
    case "offline":
      return { machine: machineScenes.offline[2] };
    case "no-slots":
      return { machine: machineScenes.noKvm[2] };
    case "disk-limit":
      return { machine: machineScenes.fourMachines[3] };
    case "no-room":
      return { machine: machineScenes.tooSmall[3] };
    case "preparing":
      return { machine: preparingAx42 };
    case "cache-unknown":
      return cacheUnknown;
    case "clear-cache-unknown":
      return confirming(cacheUnknown);
    case "docker-unknown":
      return dockerUnknown;
    case "clear-docker-unknown":
      return confirming(dockerUnknown);
    case "both-unknown":
      return bothUnknown;
    case "clear-both-unknown":
      return confirming(bothUnknown);
    case "mac":
      return { machine: macStudioMachine };
    case "mac-one-slot":
      return { machine: macMiniMachine };
    case "mac-no-virtualization":
      return { machine: macMiniOldMacos };
  }
}
