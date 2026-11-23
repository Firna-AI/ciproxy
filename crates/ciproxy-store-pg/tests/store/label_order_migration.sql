INSERT INTO workspaces (id, name) VALUES
    ('00000000-0000-0000-0000-000000000001', 'Saved limit'),
    ('00000000-0000-0000-0000-000000000002', 'No limit'),
    ('00000000-0000-0000-0000-000000000003', 'No settings row');

INSERT INTO settings (workspace_id, boat_spend_limit_usd_micros) VALUES
    ('00000000-0000-0000-0000-000000000001', 12345678),
    ('00000000-0000-0000-0000-000000000002', NULL);

INSERT INTO labels (id, workspace_id, name, vcpus, memory_gb, disk_gb, platform,
                    allow_machines, allow_boat, when_nothing_fits)
SELECT fixture.id::uuid, '00000000-0000-0000-0000-000000000001', name, 2, 8, 20,
       platform, allow_machines, allow_boat, when_nothing_fits
FROM (VALUES
    ('00000000-0000-0000-0000-000000000011', 'both', 'linux_x64', TRUE, TRUE, 'wait'),
    ('00000000-0000-0000-0000-000000000012', 'empty', 'linux_x64', FALSE, FALSE, 'wait'),
    ('00000000-0000-0000-0000-000000000013', 'macos', 'macos_arm64', TRUE, FALSE, 'wait'),
    ('00000000-0000-0000-0000-000000000014', 'machine', 'linux_x64', TRUE, FALSE, 'wait'),
    ('00000000-0000-0000-0000-000000000015', 'only-boat', 'linux_x64', FALSE, TRUE, 'largest_boat')
) AS fixture(id, name, platform, allow_machines, allow_boat, when_nothing_fits);
