# Terraform policy records

`create.json`, `update.json`, and `delete.json` contain reduced
`terraform show -json` output from Terraform 1.17.1. The recording used the
built-in `terraform_data.example` resource. Its input changed from `first`
to `second`. The delete recording used `terraform plan -destroy`.

The recording used local state only. It used no Google provider and no
cloud credentials. The full command output stays under
`.context/terraform-promotion/recording/` during this implementation.
`pool-replace.json` is a hand-worked provider-plan example in the same
Terraform JSON format. It changes the immutable federation pool ID.
It is not a recording of a GCP call. Tests also build explicit replacement
and forget inputs in the same format.

`output-only.json` records Terraform 1.17.1 JSON from a local output-only
consumer of `terraform_remote_state`. Its source state and plan used only
Terraform built-ins. It used no cloud provider or GCP call. Terraform omits
`resource_changes` for this canonical empty resource plan. The recording
protects promotion of the environment snapshot roots. Its original plan
and failing policy output stay under
`.context/ciproxy-mvp/shared-project/outputs-plan/` during implementation.
