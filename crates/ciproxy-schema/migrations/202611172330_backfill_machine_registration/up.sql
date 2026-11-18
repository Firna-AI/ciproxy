UPDATE machines AS machine
SET registered_at = credential.created_at
FROM (
    SELECT workspace_id, machine_id, MIN(created_at) AS created_at
    FROM tokens
    WHERE purpose = 'machine'
    GROUP BY workspace_id, machine_id
) AS credential
WHERE machine.workspace_id = credential.workspace_id
  AND machine.id = credential.machine_id
  AND machine.registered_at IS NULL;
