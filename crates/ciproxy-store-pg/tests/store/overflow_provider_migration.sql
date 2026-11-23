INSERT INTO workspaces (id, name)
VALUES ('00000000-0000-0000-0000-000000000001', 'Legacy workspace');
INSERT INTO settings (workspace_id, boat_spend_limit_usd_micros)
VALUES ('00000000-0000-0000-0000-000000000001', 100000000);
INSERT INTO organizations (id, workspace_id, github_id, login, installation_id)
VALUES ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001',
        40, 'example', 42);
INSERT INTO labels (id, workspace_id, name, vcpus, memory_gb, disk_gb, platform)
VALUES ('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-000000000001',
        'linux', 2, 8, 60, 'linux_x64');
INSERT INTO scale_sets (id, workspace_id, organization_id, label_id, github_id, runner_group_id)
VALUES ('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000003', 43, 41);
INSERT INTO boat_accounts (id, workspace_id, account_name, key_ciphertext, connected_at)
VALUES ('00000000-0000-0000-0000-000000000005', '00000000-0000-0000-0000-000000000001',
        'Legacy account', decode('0102', 'hex'), '2026-11-04T12:00:00Z');
INSERT INTO overflow_accounts (key_id, state_json)
VALUES ('00000000-0000-0000-0000-000000000005',
        '{"wallet":"personal","sizes":[],"trial":true,"billing_required":false,"refused_sizes":[]}');
INSERT INTO runners (id, workspace_id, organization_id, label_id, scale_set_id,
                     state, vcpus, memory_bytes, disk_bytes, boat_key_id, sandbox_id, boat_size_name)
VALUES ('00000000-0000-0000-0000-000000000006', '00000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000003',
        '00000000-0000-0000-0000-000000000004', 'busy', 2, 8000000000, 110000000000,
        '00000000-0000-0000-0000-000000000005', 'saved-sandbox', 'saved-size');
INSERT INTO jobs (id, workspace_id, organization_id, github_id, label_id, scale_set_id, runner_id,
                  name, workflow, repository, state, queued_at, started_at)
VALUES ('00000000-0000-0000-0000-000000000008', '00000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000002', 50, '00000000-0000-0000-0000-000000000003',
        '00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-000000000006',
        'test', 'test', 'example/test', 'running', '2026-11-04T12:00:00Z', '2026-11-04T12:00:00Z');
INSERT INTO metering_boat_costs (runner_id, workspace_id, starts_at, ends_at, start_usd, minute_usd)
VALUES ('00000000-0000-0000-0000-000000000006', '00000000-0000-0000-0000-000000000001',
        '2026-11-04T12:00:00Z', '2026-11-04T12:10:00Z', 0.07, 0.02);
INSERT INTO metering_boat_cursors
    (runner_id, workspace_id, starts_at, ends_at, start_usd, minute_usd, observed_at)
VALUES ('00000000-0000-0000-0000-000000000006', '00000000-0000-0000-0000-000000000001',
        '2026-11-04T12:00:00Z', '2026-11-04T12:10:00Z', 0.07, 0.02, '2026-11-04T12:15:00Z');
INSERT INTO metering_daily_boat_costs (workspace_id, day, usd)
VALUES ('00000000-0000-0000-0000-000000000001', '2026-11-04', 0.27);
INSERT INTO metering_daily_sizes
    (workspace_id, day, vcpus, memory_bytes, disk_bytes,
     machine_starts, machine_minutes, boat_starts, boat_minutes)
VALUES ('00000000-0000-0000-0000-000000000001', '2026-11-04',
        2, 8000000000, 110000000000, 0, 0, 1, 7.5);
INSERT INTO metering_job_cursors (workspace_id, organization_id, source, job_id, observed_at, facts)
VALUES ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000002',
        'actions', '00000000-0000-0000-0000-000000000008', '2026-11-04T12:07:30.000000000Z',
        '{
            "source": "actions",
            "id": "00000000-0000-0000-0000-000000000008",
            "organization_id": "00000000-0000-0000-0000-000000000002",
            "started_at": "2026-11-04T12:00:00Z",
            "completion": {"state": "running"},
            "size": {"vcpu": 2, "memory_bytes": 8000000000, "disk_bytes": 110000000000},
            "place": "boat"
        }');
