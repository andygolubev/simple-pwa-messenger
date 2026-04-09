# AGENTS.md

## Cursor Cloud specific instructions

### Project overview

This is a Terraform/Infrastructure-as-Code repository (`google-message`) intended for Google Cloud infrastructure provisioning. The `.gitignore` is configured for Terraform workflows.

### Development tools

- **Terraform** is the primary tool. It is pre-installed at `/usr/local/bin/terraform` (v1.9.8). The update script ensures it is present on each VM startup.
- No other runtime dependencies or package managers are currently needed.

### Common commands

- `terraform init` — initialize a Terraform working directory
- `terraform validate` — validate configuration files
- `terraform fmt -check` — lint/format check for `.tf` files
- `terraform plan` — preview infrastructure changes
- `terraform apply` — apply infrastructure changes

### Notes

- Since this is an IaC project, there is no traditional build/test/lint cycle. Use `terraform validate` and `terraform fmt -check` for linting, and `terraform plan` for dry-run validation.
- Google Cloud credentials are required for any real `plan`/`apply` operations against GCP. Set up authentication via `GOOGLE_APPLICATION_CREDENTIALS` environment variable or `gcloud auth application-default login`.
