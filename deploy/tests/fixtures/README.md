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
