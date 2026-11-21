import {
  lateAddWait,
  lateApprovalWait,
  lateApprovedReturn,
  multipleLateApprovals,
} from "./fixtures-late-approval.ts";
import { job, machine, setup } from "./fixtures-setup.ts";
import type { Scenario } from "./harness-scenarios.ts";

export function applyLateApprovalScenario(data: Scenario): void {
  if (data.name === "late-first-wait") {
    data.session = lateApprovalWait();
    data.signInOutcome = "late-first-completed";
  }
  if (data.name === "late-add-wait") {
    data.session = lateAddWait();
    data.signInOutcome = "late-add-completed";
  }
  if (
    data.name === "late-first-completed" ||
    data.name === "late-add-completed"
  ) {
    data.session = lateApprovedReturn();
  }
  if (data.name === "late-add-completed") {
    data.machines = [machine()];
    data.setup = setup({
      machines: 1,
      waiting_for_machine: false,
      first_job: job({
        phase: "completed",
        result: "success",
        completed_at: "2026-11-06T12:07:22Z",
      }),
    });
  }
  if (data.name === "late-multiple-completed")
    data.session = multipleLateApprovals();
}
