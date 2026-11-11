// Settings, for Admins: the labels and their sizes, the organizations with
// their repository access and the name of the workspace, and the members
// with their roles. The query names the tab. Every tab reads the settings of
// the workspace, so the Members tab counts the open requests on each tab.
// Overflow has a page of its own, so its tab address opens that page.

import { useEffect } from "react";
import { useLocalSearchParams } from "expo-router";

import { PageHeader } from "../../components/page-state.tsx";
import { overflowHref } from "../../routing/pages.ts";
import { useServices } from "../../services/context.tsx";
import { useRedirect, useValue } from "../../services/hooks.ts";
import { settingsServices } from "../../state/settings-services.ts";
import { settingsRoute, type SettingsRoute } from "../../view/settings-tabs.ts";
import {
  WorkspacePageView,
  type WorkspaceContext,
} from "../workspace-page.tsx";
import { LabelsTab } from "./labels-tab.tsx";
import { MembersTab } from "./members-tab.tsx";
import { OrganizationsBanner } from "./organizations-banner.tsx";
import { OrganizationsTab } from "./organizations-tab.tsx";
import { RepositoriesPage } from "./repositories-page.tsx";
import { SettingsTabs } from "./settings-tabs.tsx";

/** The address of the Overflow tab opens the Overflow page in its place. */
function OverflowTab({ workspaceId }: { workspaceId: string }) {
  useRedirect(overflowHref(workspaceId));
  return null;
}

function TabBody({
  context,
  route,
}: {
  context: WorkspaceContext;
  route: SettingsRoute;
}) {
  if (route.kind === "repositories") {
    return (
      <RepositoriesPage
        context={context}
        organizationId={route.organizationId}
      />
    );
  }
  switch (route.tab) {
    case "labels":
      return <LabelsTab context={context} />;
    case "overflow":
      return <OverflowTab workspaceId={context.workspace.id} />;
    case "organizations":
      return <OrganizationsTab context={context} />;
    case "members":
      return <MembersTab context={context} />;
  }
}

function Settings({
  context,
  route,
}: {
  context: WorkspaceContext;
  route: SettingsRoute;
}) {
  const store = settingsServices(useServices()).settings(context.workspace.id);
  const snapshot = useValue(store.state);
  useEffect(() => {
    void store.load();
  }, [store]);
  const openRequests =
    snapshot.status === "ready" ? snapshot.value.open_requests : null;
  return (
    <>
      <PageHeader title="Settings" />
      <SettingsTabs
        workspaceId={context.workspace.id}
        active={route.kind === "tab" ? route.tab : "organizations"}
        openRequests={openRequests}
      />
      <TabBody context={context} route={route} />
    </>
  );
}

export function SettingsScreen() {
  const params = useLocalSearchParams<{
    tab?: string | string[];
    organization?: string | string[];
  }>();
  const route = settingsRoute(params);
  const organizations = route.kind === "tab" && route.tab === "organizations";
  return (
    <WorkspacePageView
      page="settings"
      visitKey={
        route.kind === "tab"
          ? route.tab
          : `repositories:${route.organizationId}`
      }
      banner={
        organizations
          ? (context) => <OrganizationsBanner context={context} />
          : undefined
      }
    >
      {(context) => <Settings context={context} route={route} />}
    </WorkspacePageView>
  );
}
