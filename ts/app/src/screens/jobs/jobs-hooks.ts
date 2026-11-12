// The hooks that connect the Jobs screens to their stores. Each mounted page
// creates its own store for its workspace, so a page never shows another
// workspace, and every read passes through the session report.

import { useEffect, useMemo } from "react";

import { createJobsEndpoints } from "../../api/jobs.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { bannerStore } from "../../state/billing.ts";
import { createJobStore, type JobSnapshot } from "../../state/jobs-detail.ts";
import {
  createHistoryStore,
  type HistorySnapshot,
} from "../../state/jobs-history.ts";
import { createLiveStore, type LiveSnapshot } from "../../state/jobs-live.ts";
import { historyQuery, type HistoryFilters } from "../../view/jobs-links.ts";

function useJobsEndpoints() {
  const services = useServices();
  return useMemo(() => createJobsEndpoints(services.api), [services]);
}

/** The live queue, read now and again on an interval while it shows. */
export function useLive(workspaceId: string) {
  const services = useServices();
  const endpoints = useJobsEndpoints();
  const store = useMemo(
    () =>
      createLiveStore({
        read: () =>
          bannerStore(services, workspaceId).readPage(
            () => endpoints.live(workspaceId),
            (page) => page.banner,
          ),
        report: services.session.report,
        now: services.env.now,
        timers: services.env.timers,
        visibility: services.env.visibility,
      }),
    [services, endpoints, workspaceId],
  );
  useEffect(() => {
    void store.load();
    return store.watch();
  }, [store]);
  const snapshot: LiveSnapshot = useValue(store.state);
  return { snapshot, retry: () => void store.load() };
}

/** The finished jobs that match the filters of the address. */
export function useHistory(workspaceId: string, filters: HistoryFilters) {
  const services = useServices();
  const endpoints = useJobsEndpoints();
  const store = useMemo(
    () =>
      createHistoryStore({
        read: (query) =>
          bannerStore(services, workspaceId).readPage(
            () => endpoints.history(workspaceId, query),
            (page) => page.banner,
          ),
        report: services.session.report,
      }),
    [services, endpoints, workspaceId],
  );
  const { search, place, days } = filters;
  useEffect(() => {
    void store.load(historyQuery({ search, place, days }, services.env.now()));
  }, [store, services, search, place, days]);
  const snapshot: HistorySnapshot = useValue(store.state);
  const retry = () =>
    void store.load(historyQuery({ search, place, days }, services.env.now()));
  return { snapshot, retry };
}

/** One job of the workspace. */
export function useJob(workspaceId: string, jobId: string) {
  const services = useServices();
  const endpoints = useJobsEndpoints();
  const store = useMemo(
    () =>
      createJobStore({
        read: () =>
          bannerStore(services, workspaceId).readPage(
            () => endpoints.detail(workspaceId, jobId),
            (page) => page.banner,
          ),
        report: services.session.report,
      }),
    [services, endpoints, workspaceId, jobId],
  );
  useEffect(() => {
    void store.load();
  }, [store]);
  const snapshot: JobSnapshot = useValue(store.state);
  return { snapshot, retry: () => void store.load() };
}
