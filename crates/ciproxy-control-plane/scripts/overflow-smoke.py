#!/usr/bin/env python3
"""Observe real overflow, idle deletion, or the prospective spend-limit bound."""

import argparse
import os
import subprocess
import sys
import time

sys.dont_write_bytecode = True

from overflow_smoke.support import Live, sandbox_id, wait


def run(live, scenario):
    if not live.original["available"] or live.original["account"] is None:
        raise ValueError("connect Boat and configure its shared image first")
    labels = live.api("GET", "/labels")
    hold_label = next(item for item in labels if item["label"]["name"] == live.config["hold_label"])
    overflow_label = next(item for item in labels if item["label"]["name"] == live.config["overflow_label"])
    if not hold_label["label"]["allow_machines"] or hold_label["label"]["allow_boat"]:
        raise ValueError("the hold label must allow only own machines")
    if not overflow_label["label"]["allow_machines"] or not overflow_label["label"]["allow_boat"]:
        raise ValueError("the overflow label must allow both destinations")
    fit = overflow_label["fit"]
    if fit["eligible_machines"] != 1 or fit["fitting_machines"] != 1 or fit["maximum_jobs"] != "1":
        raise ValueError("the fixture needs one connected machine that fits one job")
    if any(hold_label["label"][key] != overflow_label["label"][key] for key in ["vcpus", "memory_gb", "disk_gb"]):
        raise ValueError("both fixture labels must request the same size")
    live.policy("1" if scenario == "spend-limit" else live.config["smoke_limit_cents"])
    holding = live.dispatch(live.config["hold_label"], 300)
    hold = wait(lambda: next((job for job in live.jobs(holding)
                             if job["status"] == "in_progress" and job["runner_name"]), None))
    if not live.own_runner(hold["runner_name"]):
        raise ValueError("hold job used an unexpected runner")
    demand = live.dispatch(live.config["overflow_label"], 15)
    if scenario == "spend-limit":
        until = time.monotonic() + 75
        while time.monotonic() < until:
            if any(row["dispatched"] for row in live.created()):
                raise ValueError("a delivery passed the prospective spend bound")
            if any(job["status"] == "in_progress" for job in live.jobs(demand)):
                raise ValueError("the test job did not remain queued")
            time.sleep(1)
        print("PASS: the positive limit blocks prospective overflow while demand waits")
        live.policy(live.config["smoke_limit_cents"])
    row = wait(lambda: next((row for row in live.created() if row["sandbox"]), None))
    if scenario == "idle-delete":
        if any(job["runner_name"] for job in live.jobs(demand)):
            raise ValueError("the runner already took the job; idle deletion was not proved")
        live.github("POST", "/actions/runs/" + str(demand) + "/cancel", {})
    else:
        job = wait(lambda: next((job for job in live.jobs(demand)
                                 if job["runner_name"] == "ciproxy-" + row["runner"]
                                 and job["status"] == "completed"), None))
        if job["conclusion"] != "success":
            raise ValueError("the overflow job did not succeed")
    ended = wait(lambda: next((item for item in live.created()
                               if item["delivery"] == row["delivery"] and item["phase"] == "ended"), None))
    if scenario == "idle-delete" and any(job["runner_name"] for job in live.jobs(demand)):
        raise ValueError("the runner took a job during cancellation; idle deletion was not proved")
    wait(lambda: live.ended(sandbox_id(ended)))
    if scenario != "idle-delete":
        wait(lambda: live.waited(row) is not None)
        if live.waited(row) < 30:
            raise ValueError("Boat admission preceded the configured overflow wait")
    print("PASS: the observed ciproxy runner used its sandbox and the provider confirms deletion"
          if scenario != "idle-delete" else
          "PASS: the unassigned ciproxy sandbox ended and the provider confirms deletion")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("scenario", choices=["overflow", "idle-delete", "spend-limit"])
    args = parser.parse_args()
    if not os.environ.get("CIPROXY_OVERFLOW_CREDENTIALS_FILE"):
        print("Set CIPROXY_OVERFLOW_CREDENTIALS_FILE to the private test credential file.")
        return 2
    live = None
    result = 0
    try:
        live = Live()
        run(live, args.scenario)
    except (OSError, ValueError, KeyError, TypeError, StopIteration, subprocess.SubprocessError):
        print("FAIL: live smoke could not finish. No credential or response body was printed.")
        result = 1
    finally:
        if live is not None:
            failures = live.restore()
            for failure in failures:
                print("FAIL: " + failure + " needs operator attention.")
            if failures:
                result = 1
    return result


if __name__ == "__main__":
    sys.exit(main())
