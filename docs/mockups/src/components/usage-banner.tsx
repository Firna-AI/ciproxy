// The banner that every page of the workspace shows while a soft limit is
// passed, a payment failed, or a paid plan ended. It has the words of
// api-banner.tsx, as the API gives them, so it names no plan, no passed limit,
// and no day on which a paid plan ended. One state shows one banner on every
// page, with the same title and text. At a failed payment, the payment banner
// takes the place of the soft-limit banner, as design call 42 says. On Free,
// one organization is a soft limit too, so a workspace that keeps more
// organizations after a move to Free sees the banner and the offer. On Plan
// and usage, the banner of a passed limit compares the plans, and on every
// other page it opens Plan and usage. A person who cannot pay sees the same
// banner with words in place of its action. The banner of a moment opens the
// Plan and usage and the Plans of that moment, so that its links keep the
// organizations of that moment.

import type { ReactNode } from "react";

import type { UsageScene } from "../fixtures/usage.ts";
import { LinkAction } from "./actions.tsx";
import { apiBanner } from "./api-banner.tsx";
import { paths, type CataloguePath } from "./paths.ts";

/** The page that shows the banner: Plan and usage, or any other page. */
export type BannerPage = "billing" | "other";

/** The Plan and usage and the Plans screens of one moment of the story. */
export type BillingPages = { usage: CataloguePath; plans: CataloguePath };

/** The offer of Team on Free, with the link to Plan and usage elsewhere. */
export function ChooseTeam({
  page,
  usage,
  plans = paths.plans,
}: {
  page: BannerPage;
  /** The Plan and usage screen of the state. */
  usage: CataloguePath;
  /** The Plans screen of the state. */
  plans?: CataloguePath;
}) {
  return (
    <>
      {page === "billing" ? null : (
        <LinkAction to={usage} size="sm">
          Plan and usage
        </LinkAction>
      )}
      <LinkAction to={plans} tone="primary" size="sm">
        Choose Team
      </LinkAction>
    </>
  );
}

/**
 * The Plan and usage and the Plans of each moment with a banner. The pages of
 * 8 December come from billing-scenes.tsx, which its other links share.
 */
const momentPages: Partial<Record<UsageScene, BillingPages>> = {
  "free-over": { usage: paths.usageFreeOver, plans: paths.plans },
  "free-organizations": { usage: paths.freeOrganizations, plans: paths.plans },
  "over-starts": { usage: paths.usageOverStarts, plans: paths.plansOnTeam },
  "over-machines": { usage: paths.usageOverMachines, plans: paths.plansOnTeam },
  "payment-failed": { usage: paths.paymentFailed, plans: paths.plansOnTeam },
  "payment-failed-over-limit": {
    usage: paths.paymentFailedOverLimit,
    plans: paths.plansOnTeam,
  },
  "moved-to-free": { usage: paths.movedToFree, plans: paths.plans },
};

/** The banner of a moment on a page; none for a moment without pages. */
export function usageBanner(
  scene: UsageScene,
  page: BannerPage,
  canPay = true,
  pages: BillingPages | undefined = momentPages[scene],
): ReactNode {
  if (!pages) return undefined;
  return page === "billing"
    ? apiBanner({ scene, plans: pages.plans, canPay })
    : apiBanner({ scene, other: pages, canPay });
}
