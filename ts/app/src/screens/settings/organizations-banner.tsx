// The banner of the Organizations tab. While an organization is inactive,
// it names the organization and its reasons, and offers Team or Turn on. It
// takes the place of the workspace banner of the shell, so the page keeps
// one banner. With no inactive organization, the workspace banner of the
// Organizations scope shows.

import { useState } from "react";
import { Link } from "expo-router";

import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { Notice } from "../../components/notice.tsx";
import { WorkspaceBanner } from "../../components/billing/workspace-banner.tsx";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { settingsServices } from "../../state/settings-services.ts";
import { inactiveBanner } from "../../view/settings-organization-banner.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { TurnOnDialog } from "./organization-dialogs.tsx";

export function OrganizationsBanner({
  context,
}: {
  context: WorkspaceContext;
}) {
  const store = settingsServices(useServices()).settings(context.workspace.id);
  const snapshot = useValue(store.state);
  const [turningOn, setTurningOn] = useState(false);
  const organizations =
    snapshot.status === "ready" ? snapshot.value.organizations : [];
  const banner = inactiveBanner(organizations);
  if (banner === null) {
    return (
      <WorkspaceBanner
        workspace={context.workspace}
        page="settings"
        scope="organizations"
      />
    );
  }
  const { chooseTeam, turnOn } = banner;
  const organization =
    turnOn === null
      ? undefined
      : organizations.find((item) => item.id === turnOn.id);
  return (
    <>
      <Notice
        tone="warning"
        title={banner.title}
        actions={
          chooseTeam !== null ? (
            <Link href={chooseTeam} asChild>
              <ActionLink label="Choose Team" href={chooseTeam} size="sm" />
            </Link>
          ) : turnOn !== null ? (
            <ActionButton
              label="Turn on"
              accessibilityLabel={`Turn on ${turnOn.login}`}
              tone="primary"
              size="sm"
              onPress={() => setTurningOn(true)}
            />
          ) : null
        }
      >
        {banner.body}
      </Notice>
      {turningOn && organization !== undefined ? (
        <TurnOnDialog
          organization={organization}
          turnOn={() => store.turnOn(organization.id)}
          onClose={() => setTurningOn(false)}
        />
      ) : null}
    </>
  );
}
