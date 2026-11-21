// Turn on in the banner of one inactive organization. The API offers it. The
// row of the Settings read with the same ID fills the Turn on dialog of the
// Organizations tab, which asks first and names the person who added the
// organization. Without that row, the action stays disabled, and the tab
// shows its own failed read. After a Turn on, the settings read again, and
// the watch of the Organizations scope reads the banner again.

import { useState } from "react";

import type { InactiveOrganization } from "../../api/billing-types.ts";
import { TurnOnDialog } from "../../screens/settings/organization-dialogs.tsx";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { settingsServices } from "../../state/settings-services.ts";
import { ActionButton } from "../actions.tsx";

export function TurnOnAction({
  workspaceId,
  organization,
}: {
  workspaceId: string;
  /** The organization of the API banner that the action turns on. */
  organization: InactiveOrganization;
}) {
  const store = settingsServices(useServices()).settings(workspaceId);
  const snapshot = useValue(store.state);
  const [open, setOpen] = useState(false);
  const row =
    snapshot.status === "ready"
      ? snapshot.value.organizations.find((item) => item.id === organization.id)
      : undefined;
  return (
    <>
      <ActionButton
        label="Turn on"
        accessibilityLabel={`Turn on ${organization.login}`}
        tone="primary"
        size="sm"
        disabled={row === undefined}
        onPress={() => setOpen(true)}
      />
      {open && row !== undefined ? (
        <TurnOnDialog
          organization={row}
          turnOn={() => store.turnOn(organization.id)}
          onClose={() => setOpen(false)}
        />
      ) : null}
    </>
  );
}
