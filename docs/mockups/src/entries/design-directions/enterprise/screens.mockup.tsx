import { directionScreens } from "../../../screens/directions/direction-screens.tsx";
import { enterprise } from "../../../theme/directions/enterprise.ts";

const screens = directionScreens(enterprise, {
  line: "light content beside a deep navy sidebar, like Stripe or Datadog. One blue accent, white cards with a faint shadow, and IBM Plex Sans with IBM Plex Mono.",
  rationale:
    "The navy sidebar and the white page header band give the familiar frame of an enterprise console. Each figure card shows its trend beside the value, and tables sit under a light grey header band.",
});

export const machines = screens.machines;
export const jobs = screens.jobs;
export const savings = screens.savings;
