import { directionScreens } from "../../../screens/directions/direction-screens.tsx";
import { midnight } from "../../../theme/directions/midnight.ts";

const screens = directionScreens(midnight, {
  line: "a dark infrastructure console, like Linear or Grafana. One blue accent, status colours close to GitHub dark, and Inter with JetBrains Mono.",
  rationale:
    "A dark console suits a page that stays open on a second screen. Running jobs carry a glowing dot, and each figure shows its last hour as a faint area.",
});

export const machines = screens.machines;
export const jobs = screens.jobs;
export const savings = screens.savings;
