-- Every existing label, machine, and report is Linux x64.
ALTER TABLE labels ADD COLUMN platform TEXT NOT NULL DEFAULT 'linux_x64'
    CHECK (platform IN ('linux_x64', 'macos_arm64'));
ALTER TABLE labels ALTER COLUMN platform DROP DEFAULT;

ALTER TABLE machines ADD COLUMN platform TEXT NOT NULL DEFAULT 'linux_x64'
    CHECK (platform IN ('linux_x64', 'macos_arm64'));
ALTER TABLE machines ALTER COLUMN platform DROP DEFAULT;

ALTER TABLE machines ADD COLUMN virtualization TEXT NOT NULL DEFAULT 'kvm_unavailable'
    CHECK (virtualization IN (
        'available',
        'kvm_unavailable',
        'framework_unsupported',
        'entitlement_missing',
        'operating_system_too_old'
    ));
UPDATE machines SET virtualization = 'available' WHERE kvm_available;
ALTER TABLE machines ALTER COLUMN virtualization DROP DEFAULT;
ALTER TABLE machines DROP COLUMN kvm_available;

UPDATE machine_reports
SET report = jsonb_set(
    report,
    '{hardware}',
    ((report -> 'hardware') - 'kvm') || jsonb_build_object(
        'platform', 'linux_x64',
        'virtualization',
        CASE WHEN (report #>> '{hardware,kvm}')::boolean
            THEN 'available' ELSE 'kvm_unavailable' END
    )
)
WHERE report -> 'hardware' ? 'kvm';
