import type { PagesSmokeRun } from "./browser-pages-runtime.ts";
import { jobsScreenRuns } from "./browser-pages-jobs-screens.ts";
import { savingsScreenRuns } from "./browser-pages-savings-screens.ts";
import { billingScreenRuns } from "./browser-pages-billing-screens.ts";
import { loadingRuns } from "./browser-pages-loading.ts";
import { dataJourneys } from "./browser-pages-data-journeys.ts";
import { paymentJourneys } from "./browser-pages-payment-journeys.ts";
import { capabilityRuns } from "./browser-pages-capabilities.ts";
import { billingJourneys } from "./browser-pages-billing-journeys.ts";
import { bannerJourneys } from "./browser-pages-banner-journeys.ts";
import { billingErrorRuns } from "./browser-pages-error-journeys.ts";

export function createPagesRegistry(
  origin: string,
  output: string,
): readonly PagesSmokeRun[] {
  return [
    ...jobsScreenRuns(origin),
    ...savingsScreenRuns(origin),
    ...billingScreenRuns(origin),
    ...loadingRuns(origin),
    ...dataJourneys(origin, output),
    ...paymentJourneys(origin, output),
    ...capabilityRuns(origin, output),
    ...billingJourneys(origin, output),
    ...bannerJourneys(origin, output),
    ...billingErrorRuns(origin, output),
  ];
}
