import { directionScreens } from "../../../screens/directions/direction-screens.tsx";
import { graphite } from "../../../theme/directions/graphite.ts";

const screens = directionScreens(graphite, {
  line: "light and monochrome, like Vercel. Black is the one accent, colour marks status only, and the type is Geist with Geist Mono.",
  rationale:
    "Monochrome chrome leaves colour to the status dots, so a problem stands out at once. The top bar holds the organization and the page as breadcrumbs, and the figures sit in one divided strip.",
});

export const machines = screens.machines;
export const jobs = screens.jobs;
export const savings = screens.savings;
