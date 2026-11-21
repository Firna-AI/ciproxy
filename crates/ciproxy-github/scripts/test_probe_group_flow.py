"""Offline regression for delayed runner-group job metadata."""

from types import SimpleNamespace
import unittest

from probe_common import ProbeError
from probe_group_flow import check_jobs


class FakeApp:
    def __init__(self, allowed, refused):
        self.allowed_job = {"status": "in_progress", "conclusion": None, "runner_id": 37}
        self.refused_job = {"status": "queued", "conclusion": None,
                            "runner_id": None, "runner_name": None}
        self.run = {"status": "completed", "conclusion": "success"}
        self.responses = {}
        for fixture, job in ((allowed, self.allowed_job), (refused, self.refused_job)):
            path = fixture.prefix + "/actions/runs/" + str(fixture.run_id)
            self.responses[path + "/jobs?per_page=100"] = (200, {"jobs": [job]})
        path = allowed.prefix + "/actions/runs/" + str(allowed.run_id)
        self.responses[path] = (200, self.run)

    def call(self, path):
        return self.responses[path]


class RunnerGroupJobsTests(unittest.TestCase):
    def test_waits_for_final_job_metadata_before_accepting_success(self):
        allowed = SimpleNamespace(prefix="/repos/probe/allowed", run_id=101)
        refused = SimpleNamespace(prefix="/repos/probe/refused", run_id=102)
        app = FakeApp(allowed, refused)

        try:
            self.assertIs(check_jobs(app, allowed, refused, 37), False)
            app.allowed_job["status"] = "completed"
            self.assertIs(check_jobs(app, allowed, refused, 37), False)
        except ProbeError as error:
            self.fail(f"A successful run must wait for final job metadata: {error}")

        app.allowed_job["conclusion"] = "success"
        self.assertIs(check_jobs(app, allowed, refused, 37), True)

        app.allowed_job["conclusion"] = "failure"
        with self.assertRaisesRegex(ProbeError, "^The allowed repository probe job did not succeed$"):
            check_jobs(app, allowed, refused, 37)

        app.run["conclusion"] = "failure"
        app.allowed_job.update(status="in_progress", conclusion=None)
        with self.assertRaisesRegex(ProbeError, "^The allowed repository probe job did not succeed$"):
            check_jobs(app, allowed, refused, 37)

        app.run["conclusion"] = "success"
        app.allowed_job.update(status="completed", conclusion="success")
        app.refused_job["runner_id"] = 37
        with self.assertRaisesRegex(ProbeError, "^The refused repository job left the queue or received a runner$"):
            check_jobs(app, allowed, refused, 37)


if __name__ == "__main__":
    unittest.main()
