import type {
  JobOverflow,
  JobsLivePage,
  WaitingLabelFit,
} from "../src/api/jobs-types.ts";
import type { ProviderLabelFit, ProviderSizeFit } from "../src/api/schema.ts";
import { boatPlace, machinePlace } from "../scripts/m20-fixtures-labels.ts";
import { fixtureJob } from "../scripts/fixtures-jobs.ts";
import { liveJobs } from "../scripts/fixtures-jobs-live.ts";

/** The browser time of each read in the notice tests. */
export const readAt = Date.parse("2026-11-06T12:00:00Z");

/**
 * Whether the own machines run the label: one fits it, none fits it, none
 * takes jobs, or the label does not run on them.
 */
export type OwnFacts = "fits" | "none" | "no-machine" | "forbidden";

export type LabelExtra = {
  size?: { vcpus: number; memory_gb: number; disk_gb: number };
  whenNothingFits?: "wait" | "largest_overflow";
};

/**
 * A waiting label with its own fit and its Boat fit. Its `nothing_fits`
 * follows the rule of the API: false when a place fits, true when every
 * allowed place is known not to fit.
 */
export function waitingLabel(
  name: string,
  jobs: number,
  own: OwnFacts,
  boat: ProviderSizeFit,
  extra: LabelExtra = {},
): WaitingLabelFit {
  const size = extra.size ?? { vcpus: 8, memory_gb: 16, disk_gb: 60 };
  const fits = own === "fits";
  const boatFits =
    boat.state === "fitting" || boat.state === "largest_fallback";
  const boatOut = boat.state === "disabled" || boat.state === "no_size_fits";
  const providers: ProviderLabelFit[] = [{ provider: "boat", fit: boat }];
  const places = [
    ...(fits ? [machinePlace] : []),
    ...(boatFits ? [boatPlace] : []),
  ];
  return {
    label_id: `label-${name}`,
    label_name: name,
    waiting_jobs: String(jobs),
    can_edit: true,
    ask_admin: false,
    nothing_fits: fits || boatFits ? false : boatOut ? true : null,
    current: {
      id: `label-${name}`,
      label: {
        name,
        ...size,
        order: [
          ...(own === "forbidden" ? [] : [machinePlace]),
          ...(boat.state === "disabled" ? [] : [boatPlace]),
        ],
        when_nothing_fits:
          extra.whenNothingFits ??
          (boat.state === "largest_fallback" ? "largest_overflow" : "wait"),
      },
      fit: {
        eligible_machines: own === "no-machine" ? 0 : 3,
        fitting_machines: fits ? 3 : 0,
        no_machine_fits: !fits,
        unavailable_machines: 0,
        minimum_jobs: fits ? "2" : null,
        maximum_jobs: fits ? "2" : null,
        providers,
        machines: [],
        fitting_places: places,
        routing:
          places.length > 0
            ? { kind: "places", places }
            : { kind: "waits", reason: { kind: "nothing_fits" } },
      },
    },
    providers,
  };
}

export const connectedBoat: JobOverflow = {
  provider: "boat",
  connected: true,
  key_rejected: false,
  billing_required: false,
  retry_at: null,
  limit_cents: "10000",
  counted_cents: "1200",
  limit_reached: false,
  measured_complete: true,
  next_month: "2026-12-01T00:00:00Z",
  refused_sizes: [],
};

export const fitting = {
  state: "fitting",
  size: { vcpus: 8, memory_bytes: "16000000000", disk_bytes: "120000000000" },
  size_name: "large",
  refused: false,
} satisfies ProviderSizeFit;

export const largest = {
  state: "largest_fallback",
  size: { vcpus: 16, memory_bytes: "64000000000", disk_bytes: "240000000000" },
  size_name: "xlarge",
  refused: false,
} satisfies ProviderSizeFit;

/** The live queue with these waiting labels and `used` of 6 own slots. */
export function queue(
  labels: WaitingLabelFit[],
  overflow: JobOverflow,
  used = 6,
): JobsLivePage {
  const base = liveJobs("live", "admin", null);
  const count = labels.reduce((sum, fit) => sum + Number(fit.waiting_jobs), 0);
  const job = fixtureJob({ status: "queued", location: null });
  return {
    ...base,
    own_slots_total: "6",
    own_slots_used: String(used),
    waiting: Array.from({ length: count }, (_, i) => ({
      ...job,
      id: `waiting-${i}`,
    })),
    waiting_count: String(count),
    waiting_labels: labels,
    overflow: [overflow],
  };
}
