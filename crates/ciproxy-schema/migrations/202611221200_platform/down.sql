-- macOS rows have no Linux form, so remove them before the old schema returns.
DELETE FROM machine_reports
WHERE report #>> '{hardware,platform}' IS DISTINCT FROM 'linux_x64';
UPDATE machine_reports
SET report = jsonb_set(
    report,
    '{hardware}',
    ((report -> 'hardware') - 'platform' - 'virtualization') || jsonb_build_object(
        'kvm', report #>> '{hardware,virtualization}' = 'available'
    )
);

ALTER TABLE machines ADD COLUMN kvm_available BOOLEAN NOT NULL DEFAULT FALSE;
UPDATE machines SET kvm_available = virtualization = 'available';
ALTER TABLE machines ALTER COLUMN kvm_available DROP DEFAULT;
ALTER TABLE machines DROP COLUMN virtualization;
ALTER TABLE machines DROP COLUMN platform;

ALTER TABLE labels DROP COLUMN platform;
