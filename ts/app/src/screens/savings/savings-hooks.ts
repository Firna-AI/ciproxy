// The hook that connects the Savings screen to its store. The mounted page
// creates its own store for its workspace, reads the range of the address,
// and stops reading again when it closes.

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
  useEffect(() => {
    void store.select(range);
  }, [store, range]);
  useEffect(() => () => store.stop(), [store]);
  const snapshot: SavingsSnapshot = useValue(store.state);
  return { snapshot, retry: () => void store.select(range) };
}
