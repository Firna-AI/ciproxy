// The banner of the Organizations scope after a change of the organizations.
// The scope reads its banner at each entry, also when a read of it just
// ended, so a change that another page made always shows. While the scope
// shows, a Settings read that changes the organizations or their holds reads
// the banner again. Turn on and Take out read the settings again after they
// succeed, from the banner or from a row. The API still chooses the banner:
// a Settings read only starts a new read of it.

import type { AppServices } from "../services/services.ts";
import { bannerStore } from "./billing.ts";
import { settingsServices } from "./settings-services.ts";
import type { SettingsSnapshot } from "./settings-store.ts";

/**
 * What the banner of the scope depends on in a Settings read: each
 * organization with its state and its activation, in the order of the read.
 * Null while the read has no value.
 */
function holdKey(snapshot: SettingsSnapshot): string | null {
  if (snapshot.status !== "ready") return null;
  return JSON.stringify(
    snapshot.value.organizations.map((item) => [
      item.id,
      item.login,
      item.state,
      item.activation,
    ]),
  );
}

/**
 * Open the banner of the Organizations scope: read it now, then read it again
 * after each Settings read whose organizations or holds differ from the last
 * one. The first value of a Settings read that was still loading only sets
 * the comparison. Call the returned function when the scope closes.
 */
export function watchOrganizationHolds(
  services: AppServices,
  workspaceId: string,
): () => void {
  const settings = settingsServices(services).settings(workspaceId);
  const banner = bannerStore(services, workspaceId, "organizations");
  let seen = holdKey(settings.state.get());
  void banner.refresh();
  return settings.state.subscribe(() => {
    const next = holdKey(settings.state.get());
    if (next === null || next === seen) return;
    const first = seen === null;
    seen = next;
    if (!first) void banner.refresh();
  });
}
