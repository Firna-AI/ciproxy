// The hook that connects the Savings screen to its store. The mounted page
// creates its own store for its workspace, opens its lifetime before the
// first read, and reads the range of the address. The cleanup of that
// lifetime runs when the page closes, so a read that answers later starts
// no new refresh.

import { useEffect, useMemo } from "react";

import type { SavingsRange } from "../../api/savings-types.ts";
import { createSavingsEndpoints } from "../../api/savings.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import {
  createSavingsStore,
  type SavingsSnapshot,
} from "../../state/savings.ts";

export function useSavings(workspaceId: string, range: SavingsRange) {
  const services = useServices();
  const store = useMemo(() => {
    const endpoints = createSavingsEndpoints(services.api);
    return createSavingsStore({
      read: (asked) => endpoints.report(workspaceId, asked),
      report: services.session.report,
      timers: services.env.timers,
      visibility: services.env.visibility,
    });
  }, [services, workspaceId]);
  // The lifetime opens first, so its cleanup exists before any read answers.
  useEffect(() => store.open(), [store]);
  useEffect(() => {
    void store.select(range);
  }, [store, range]);
  const snapshot: SavingsSnapshot = useValue(store.state);
  return { snapshot, retry: () => void store.select(range) };
}
