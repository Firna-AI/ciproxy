// A billing route: its page in the shell, for the workspace of its query.
// Stripe returns to one URL for all workspaces, so a Stripe return without a
// workspace goes on with the workspace that this tab saved before it left
// for Stripe, and with no other. A Checkout return with nothing saved says so
// in place of guessing a workspace; the API still checks the exact session.

import { useEffect, useRef, useState, type ReactNode } from "react";
import { useLocalSearchParams } from "expo-router";

import { ActionLink } from "../../components/actions.tsx";
import { AuthShell } from "../../components/auth-shell.tsx";
import { InfoIcon } from "../../components/icons.tsx";
import { SessionWaiting, StateBlock } from "../../components/page-state.tsx";
import { idParam, pages } from "../../routing/pages.ts";
import { useServices } from "../../services/context.tsx";
import { stripeReturnTarget } from "../../state/billing-flows.ts";
import {
  clearPaymentContext,
  type PaymentContextKind,
} from "../../state/billing-context.ts";
import type { BillingPath } from "../../view/billing-routes.ts";
import {
  WorkspacePageView,
  type WorkspaceContext,
} from "../workspace-page.tsx";

type Query = Record<string, string | string[]>;

function ReturnWithoutWorkspace() {
  return (
    <AuthShell>
      <StateBlock
        icon={InfoIcon}
        title="This tab cannot tell the workspace of this payment"
        action={
          <ActionLink label="Open ciproxy" href={pages.home} tone="primary" />
        }
      >
        Stripe also tells ciproxy about the payment, so Plan and usage of its
        workspace shows the plan once Stripe confirms it.
      </StateBlock>
    </AuthShell>
  );
}

export function BillingRoute({
  path,
  stripeReturn,
  children,
}: {
  path: BillingPath | "/billing";
  /** The saved Stripe page whose return comes to this route, if any. */
  stripeReturn: PaymentContextKind | null;
  children: (context: WorkspaceContext) => ReactNode;
}) {
  const services = useServices();
  const query = useLocalSearchParams<Query>();
  const workspace = idParam(query.workspace);
  const [unsaved, setUnsaved] = useState(false);
  const sent = useRef(false);
  useEffect(() => {
    if (workspace !== null || sent.current) return;
    const href =
      stripeReturn === null
        ? null
        : stripeReturnTarget(services, stripeReturn, path, query);
    if (href === null) {
      setUnsaved(true);
      return;
    }
    sent.current = true;
    if (stripeReturn === "portal") {
      clearPaymentContext(services.env.tab, "portal");
    }
    services.navigator.go(href, { replace: true });
  }, [workspace, stripeReturn, path, query, services]);

  if (workspace === null && !unsaved) {
    return <SessionWaiting snapshot={{ status: "loading" }} />;
  }
  if (workspace === null && path === "/checkout-return") {
    return <ReturnWithoutWorkspace />;
  }
  return <WorkspacePageView page="billing">{children}</WorkspacePageView>;
}
