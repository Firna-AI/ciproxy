DROP TRIGGER scale_demand_job_usage_cleanup ON github_jobs;
DROP TRIGGER scale_attempt_job_usage_cleanup ON github_job_attempts;
DROP TRIGGER actions_job_usage_cleanup ON jobs;
DROP FUNCTION remove_scale_job_usage();
DROP FUNCTION remove_actions_job_usage();
DROP FUNCTION runner_has_job_usage_history(runners);
DROP INDEX runners_scale_set_github_id;
DROP TABLE job_usage_steps;
DROP TABLE job_usage_samples;
