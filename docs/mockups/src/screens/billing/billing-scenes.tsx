// The billing pages of the moments of the story. On 8 December, mokly-ai is
// on Free with mokly-ai and mokly-labs, and mokly-studio joins inactive. Every
// billing link of that day opens the Plan and usage and the Plans of that
// day, so that each page keeps the three organizations: the banner, the pace
// notice, and the Plan panel of Plan and usage, the breadcrumb of Plans, and
// the banner and the row of the inactive organization. The moments of
// September and 4 December keep the pages that they open.

import type { ReactNode } from "react";

import { paths, type CataloguePath } from "../../components/paths.ts";
import {
  usageBanner,
  type BannerPage,
  type BillingPages,
} from "../../components/usage-banner.tsx";
import { usageScenes, type UsageScene } from "../../fixtures/usage.ts";

/** The use of 8 December, with mokly-studio inactive. */
export const inactiveUse = usageScenes["inactive-organization"];

/** The Plan and usage and the Plans of 8 December. */
export const inactivePages: BillingPages = {
  usage: paths.usageInactive,
  plans: paths.plansInactive,
};

/** The Plans screen that the links of a moment on Free open. */
export function freePlansOf(scene: UsageScene): CataloguePath {
  return scene === "inactive-organization" ? inactivePages.plans : paths.plans;
}

/** The banner of a moment, on every page but the Organizations tab. */
export function sceneBanner(
  scene: UsageScene,
  page: BannerPage,
  canPay: boolean,
): ReactNode {
  return scene === "inactive-organization"
    ? usageBanner(scene, page, canPay, inactivePages)
    : usageBanner(scene, page, canPay);
}
