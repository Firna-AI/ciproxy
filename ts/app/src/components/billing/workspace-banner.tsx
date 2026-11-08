// The one banner of every workspace page: a failed payment, an ended paid
// plan, or a passed limit. Jobs, Savings, Plan and usage, and Plans take it
// from their own page reads, and so do Checkout for a payer with a valid
// choice and its return for a payer with a session. Every other view, and the
// Organizations scope, reads GET /banner at each page change. A failed
// endpoint read shows itself with Try again, so a payment problem is never
// hidden. Update card opens the Stripe billing portal. The banner gets the
// path of the page, so that on Plans it does not link to Plans.

import { useEffect, useMemo } from "react";
import { Link, useLocalSearchParams, usePathname } from "expo-router";

import type { BannerScope } from "../../api/banner.ts";
import type { WorkspaceView } from "../../api/schema.ts";
import type { WorkspacePage } from "../../routing/pages.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { bannerStore, payerRole } from "../../state/billing.ts";
import { createPortalAction } from "../../state/billing-flows.ts";
import { bannerModel, type BannerActionModel } from "../../view/banner.ts";
import {
  billingPaths,
  checkoutQuery,
  sessionIdQuery,
} from "../../view/billing-routes.ts";
import { ActionButton, ActionLink } from "../actions.tsx";
import { ExternalIcon } from "../icons.tsx";
import { Notice } from "../notice.tsx";
import { RoleNote } from "../role-note.tsx";

function BannerAction({
  action,
  onPortal,
  busy,
}: {
  action: BannerActionModel;
  onPortal: () => void;
  busy: boolean;
}) {
  if (action.kind === "ask") return <RoleNote>{action.text}</RoleNote>;
  if (action.kind === "portal") {
    return (
      <ActionButton
        label={action.label}
        tone="primary"
        size="sm"
        icon={ExternalIcon}
        busy={busy}
        onPress={onPortal}
      />
    );
  }
  return (
    <Link href={action.href} asChild>
      <ActionLink
        label={action.label}
        href={action.href}
        tone="primary"
        size="sm"
      />
    </Link>
  );
}

type Query = Record<string, string | string[]>;

/**
 * Whether the view reads GET /banner: every view whose page read does not
 * return the banner. Checkout reads its review only for a payer with a valid
 * choice, and the return checks its session only for a payer with one.
 */
function readsEndpoint(
  scope: BannerScope,
  page: WorkspacePage,
  path: string,
  query: Query,
  payer: boolean,
): boolean {
  if (scope !== "page") return true;
  if (page === "jobs" || page === "savings") return false;
  if (page !== "billing") return true;
  if (path === billingPaths.checkout) {
    return !payer || checkoutQuery(query) === null;
  }
  if (path === billingPaths.checkoutReturn) {
    return !payer || sessionIdQuery(query.session_id) === null;
  }
  return false;
}

export function WorkspaceBanner({
  workspace,
  page,
  scope = "page",
}: {
  workspace: WorkspaceView;
  page: WorkspacePage;
  /** The Organizations tab asks for its own scope; every page else "page". */
  scope?: BannerScope;
}) {
  const services = useServices();
  const path = usePathname();
  const query = useLocalSearchParams<Query>();
  const store = bannerStore(services, workspace.id, scope);
  const snapshot = useValue(store.state);
  const portal = useMemo(
    () => createPortalAction(services, workspace.id),
    [services, workspace.id],
  );
  const opening = useValue(portal.state);
  const endpoint = readsEndpoint(
    scope,
    page,
    path,
    query,
    payerRole(workspace.role ?? null),
  );
  useEffect(() => {
    if (endpoint) void store.visit();
  }, [store, page, path, endpoint]);

  const failure =
    endpoint &&
    (snapshot.status === "failed" ||
      (snapshot.status === "ready" && snapshot.refreshFailure !== null));
  const banner = snapshot.status === "ready" ? snapshot.banner : null;
  const model =
    banner === null
      ? null
      : bannerModel(
          banner,
          { id: workspace.id, name: workspace.name },
          page,
          path,
        );
  return (
    <>
      {model ? (
        <Notice
          tone={model.tone}
          title={model.title}
          actions={
            model.actions.length === 0
              ? undefined
              : model.actions.map((action) => (
                  <BannerAction
                    key={action.kind === "ask" ? action.text : action.label}
                    action={action}
                    busy={opening.status === "working"}
                    onPortal={() => void portal.run()}
                  />
                ))
          }
        >
          {model.body}
        </Notice>
      ) : null}
      {opening.status === "failed" ? (
        <Notice tone="danger" title="Unable to open the billing portal">
          Check the connection, then try again.
        </Notice>
      ) : null}
      {failure ? (
        <Notice
          tone="info"
          title="Unable to check the plan status"
          actions={
            <ActionButton
              label="Try again"
              size="sm"
              onPress={() => void store.refresh()}
            />
          }
        >
          Plan and usage shows the plan when the check works again.
        </Notice>
      ) : null}
    </>
  );
}
