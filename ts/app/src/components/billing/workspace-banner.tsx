// The one banner of every workspace page, as GET /banner chooses it: a failed
// payment, an ended paid plan, or a passed limit. Each page change checks it
// again. A failed check shows itself with Try again, so a payment problem is
// never hidden. Update card opens the Stripe billing portal. The banner gets
// the path of the page, so that on Plans it does not link to Plans.

import { useEffect, useMemo } from "react";
import { Link, usePathname } from "expo-router";

import type { BannerScope } from "../../api/banner.ts";
import type { WorkspaceView } from "../../api/schema.ts";
import type { WorkspacePage } from "../../routing/pages.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { bannerStore } from "../../state/billing.ts";
import { createPortalAction } from "../../state/billing-flows.ts";
import { bannerModel, type BannerActionModel } from "../../view/banner.ts";
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
  const store = bannerStore(services, workspace.id, scope);
  const snapshot = useValue(store.state);
  const portal = useMemo(
    () => createPortalAction(services, workspace.id),
    [services, workspace.id],
  );
  const opening = useValue(portal.state);
  useEffect(() => {
    void store.visit();
  }, [store, page]);

  const failure =
    snapshot.status === "failed" ||
    (snapshot.status === "ready" && snapshot.refreshFailure !== null);
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
