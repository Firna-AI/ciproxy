// The notices of the live queue: each reason why jobs wait that the API
// reports. A notice names only the facts that the API gives. An action that
// changes a setting goes to a caller that the API says may manage it, and
// everyone else reads "Ask an Admin". A link to GitHub is for everyone,
// because GitHub decides who may act there.

import type { JobsLivePage, WaitingLabelFit } from "../api/jobs-types.ts";
import type { NoticeTone } from "./page-model.ts";
import { addUtcDays, formatCents, formatUtcDate } from "./exact-format.ts";
import { formatGigabytes, formatTime } from "./format.ts";

export type NoticeAction =
  | { kind: "go"; label: string; href: string }
  | { kind: "leave"; label: string; href: string }
  | { kind: "ask"; text: string };

export type LiveNotice = {
  key: string;
  tone: NoticeTone;
  title: string;
  body: string;
  action: NoticeAction | null;
};

const askAdmin: NoticeAction = { kind: "ask", text: "Ask an Admin" };

const monthName = new Intl.DateTimeFormat("en-GB", {
  month: "long",
  timeZone: "UTC",
});

function jobs(count: number | bigint): string {
  return BigInt(count) === 1n ? "1 job" : `${count} jobs`;
}

/** "when it has waited 30 seconds", or "at once": the saved overflow wait. */
function waitText(seconds: number): string {
  if (seconds === 0) return "at once";
  if (seconds % 60 === 0) {
    const minutes = seconds / 60;
    return `when it has waited ${minutes} ${minutes === 1 ? "minute" : "minutes"}`;
  }
  return `when it has waited ${seconds} seconds`;
}

function labelNotices(fit: WaitingLabelFit, settings: string): LiveNotice[] {
  const count = BigInt(fit.waiting_jobs);
  const verb = count === 1n ? "waits" : "wait";
  const action: NoticeAction | null = fit.can_edit
    ? { kind: "go", label: "Edit label", href: settings }
    : fit.ask_admin
      ? askAdmin
      : null;
  const size = fit.current?.label ?? null;
  const asked = size ? `${size.vcpus} vCPU and ${size.memory_gb} GB` : null;
  const notices: LiveNotice[] = [];
  if (fit.nothing_fits === true) {
    notices.push({
      key: `fits-${fit.label_id}`,
      tone: "warning",
      title: `${jobs(count)} with the label ${fit.label_name} ${verb}`,
      body: asked
        ? `No machine and no Boat size has ${asked}. Change the label, or let it run on the largest Boat size.`
        : "No machine and no Boat size fits this label.",
      action,
    });
  } else if (fit.nothing_fits === null || fit.nothing_fits === undefined) {
    notices.push({
      key: `fit-unknown-${fit.label_id}`,
      tone: "info",
      title: `${jobs(count)} with the label ${fit.label_name} ${verb}`,
      body: "Some machine or Boat facts are not available now, so ciproxy cannot tell where this label fits. The jobs start as soon as a machine or Boat can run them.",
      action: null,
    });
  }
  if (fit.boat.state === "largest_fallback") {
    const boat = `Boat ${fit.boat.size_name}, with ${fit.boat.size.vcpus} vCPU and ${formatGigabytes(fit.boat.size.memory_bytes)}`;
    notices.push({
      key: `largest-${fit.label_id}`,
      tone: "info",
      title: `Jobs with the label ${fit.label_name} start on the largest Boat size`,
      body: asked
        ? `No machine and no Boat size has ${asked}, so ciproxy starts them on ${boat}.`
        : `ciproxy starts them on ${boat}.`,
      action,
    });
  }
  return notices;
}

function boatNotices(page: JobsLivePage, settings: string): LiveNotice[] {
  const overflow = page.overflow ?? null;
  if (overflow === null) return [];
  const manage = (label: string): NoticeAction =>
    page.can_manage ? { kind: "go", label, href: settings } : askAdmin;
  const notices: LiveNotice[] = [];
  if (overflow.limit_reached === true) {
    const until = overflow.next_month.slice(0, 10);
    const month = monthName.format(new Date(addUtcDays(until, -1)));
    const limit = overflow.limit_cents ?? null;
    notices.push({
      key: "spend-limit",
      tone: "warning",
      title: limit
        ? `Boat reached its ${formatCents(limit)} limit for ${month}`
        : `Boat reached its spend limit for ${month}`,
      body: `Jobs run on your machines only, and wait there for a free slot. Boat takes jobs again on ${formatUtcDate(until)}.`,
      action: manage("Raise the limit"),
    });
  }
  if (overflow.key_rejected) {
    notices.push({
      key: "boat-key",
      tone: "warning",
      title: "Boat rejected the key",
      body: "Jobs run on your machines only, and wait there for a free slot, until a new key is connected.",
      action: manage("Connect a new key"),
    });
  }
  if (overflow.billing_required) {
    notices.push({
      key: "boat-billing",
      tone: "warning",
      title: "Boat needs billing on your Boat account",
      body: "Boat starts no new sandboxes until its billing is set up. Jobs run on your machines only, and wait there for a free slot.",
      action: null,
    });
  }
  const refused =
    (overflow.retry_at ?? null) !== null ||
    overflow.refused_sizes.length > 0 ||
    page.waiting_labels.some(
      (fit) => "refused" in fit.boat && fit.boat.refused,
    );
  if (refused) {
    const retry = overflow.retry_at ?? null;
    notices.push({
      key: "boat-refused",
      tone: "warning",
      title: "Boat refused new sandboxes",
      body: retry
        ? `Waiting jobs take the first free slot on your machines, and ciproxy tries Boat again at ${formatTime(retry)}.`
        : "Waiting jobs take the first free slot on your machines, and ciproxy tries Boat again.",
      action: null,
    });
  }
  return notices;
}

function fullNotice(page: JobsLivePage, settings: string, boat: boolean) {
  const used = page.own_slots_used ?? null;
  const total = page.own_slots_total ?? null;
  const waiting = page.waiting.length;
  if (waiting === 0 || used === null || total === null) return null;
  if (BigInt(total) === 0n || BigInt(used) < BigInt(total)) return null;
  const manage = (label: string): NoticeAction =>
    page.can_manage ? { kind: "go", label, href: settings } : askAdmin;
  const lead = `${jobs(waiting)} ${waiting === 1 ? "waits" : "wait"} for a free slot.`;
  const notice = (body: string, action: NoticeAction | null): LiveNotice => ({
    key: "machines-full",
    tone: "info",
    title: "Your machines are full",
    body: `${lead} ${body}`,
    action,
  });
  if (!page.overflow_enabled) {
    return notice(
      "Boat overflow is off, so each one starts when a slot is free.",
      manage("Turn on Boat"),
    );
  }
  if (page.overflow && !page.overflow.connected) {
    return notice(
      "Boat is not connected, so each one starts when a slot is free.",
      manage("Connect Boat"),
    );
  }
  if (boat) return null;
  return notice(
    `Each one starts on Boat ${waitText(page.overflow_wait_seconds)}.`,
    null,
  );
}

export function liveNotices(
  page: JobsLivePage,
  settingsHref: string,
): LiveNotice[] {
  const notices: LiveNotice[] = page.visibility.map((item) =>
    item.suspended
      ? {
          key: `suspended-${item.organization}`,
          tone: "warning" as const,
          title: `The GitHub App is suspended on ${item.organization}`,
          body: `New jobs of ${item.organization} wait on GitHub until an owner unsuspends it, and running jobs finish.`,
          action: {
            kind: "leave",
            label: "Open GitHub",
            href: item.github_url,
          },
        }
      : {
          key: `removed-${item.organization}`,
          tone: "warning" as const,
          title: `The GitHub App was removed from ${item.organization}`,
          body: `Jobs of ${item.organization} no longer run on ciproxy. An owner can install the App again to keep its job history.`,
          action: {
            kind: "leave",
            label: "Install the App",
            href: item.github_url,
          },
        },
  );
  const boat = boatNotices(page, settingsHref);
  notices.push(...boat);
  for (const fit of page.waiting_labels) {
    notices.push(...labelNotices(fit, settingsHref));
  }
  const full = fullNotice(page, settingsHref, boat.length > 0);
  if (full) notices.push(full);
  return notices;
}
