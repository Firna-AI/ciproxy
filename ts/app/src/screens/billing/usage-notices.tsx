// The notices that only Plan and usage shows: a payment that Stripe confirms,
// the day on which a paid plan ends after Move to Free, and the day on which
// the pace passes the plan, with the offer that covers the use. A payment
// that this tab can continue links to its own Stripe page. A person who
// cannot pay reads who can in place of each action.

import { Link } from "expo-router";

import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { ExternalIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import { RoleNote } from "../../components/role-note.tsx";
import { askPayer } from "../../view/billing-format.ts";
import { plansHref } from "../../view/billing-routes.ts";
import type { OpenPayment } from "../../state/billing-open-payment.ts";
import type { UsageNotice } from "../../view/billing-usage.ts";

/** Continue a payment on its own Stripe page; an invoice page opens a tab. */
export function ContinuePayment({ resume }: { resume: OpenPayment }) {
  return (
    <ActionLink
      label="Continue payment"
      href={resume.url}
      tone="primary"
      size="sm"
      icon={ExternalIcon}
      newTab={resume.kind === "bank"}
    />
  );
}

export function UsageNotices({
  notices,
  workspaceId,
  canPay,
  onAddOns,
  onPortal,
}: {
  notices: readonly UsageNotice[];
  workspaceId: string;
  canPay: boolean;
  /** Set the add-ons to the ones that cover the use. */
  onAddOns: () => void;
  onPortal: () => void;
}) {
  return notices.map((notice) => {
    const ask = canPay ? null : <RoleNote>{askPayer}</RoleNote>;
    if (notice.kind === "confirming") {
      return (
        <Notice
          key={notice.kind}
          tone="info"
          title={notice.title}
          actions={
            notice.resume === null
              ? undefined
              : (ask ?? <ContinuePayment resume={notice.resume} />)
          }
        >
          {notice.body}
        </Notice>
      );
    }
    if (notice.kind === "ends") {
      return (
        <Notice
          key={notice.kind}
          tone="info"
          title={notice.title}
          actions={
            ask ?? (
              <ActionButton
                label={notice.keep}
                size="sm"
                icon={ExternalIcon}
                onPress={onPortal}
              />
            )
          }
        >
          {notice.body}
        </Notice>
      );
    }
    const plans = plansHref(workspaceId);
    return (
      <Notice
        key={notice.kind}
        tone="warning"
        title={notice.title}
        actions={
          ask ??
          (notice.action === "add-ons" ? (
            <ActionButton
              label="Review add-ons"
              tone="primary"
              size="sm"
              onPress={onAddOns}
            />
          ) : (
            <Link href={plans} asChild>
              <ActionLink
                label="Compare plans"
                href={plans}
                tone="primary"
                size="sm"
              />
            </Link>
          ))
        }
      >
        {notice.body}
      </Notice>
    );
  });
}
