# Foundation Module Integration Notes

This repository follows OpenSpec guidance to prefer Google foundation modules where they fit the workload.

## Reused foundation layers

- **terraform-example-foundation (TEF)**: Used as the reference baseline pattern for organization/folder/project landing-zone controls. Those controls are intentionally kept outside this workload stack to avoid duplicating enterprise baseline responsibilities in app IaC.
- **cloud-foundation-fabric (CFF)**: Used as the workload design style for composable project services, IAM, Firestore, Secret Manager, and function-adjacent resources.

## Custom layers in this repository

For this change, modules under `infra/modules/*` are implemented locally to keep the encrypted messenger stack self-contained and directly aligned to `ARCHITECTURE.md`:

- `project-services`
- `iam`
- `identity-platform`
- `firestore`
- `secret-manager`
- `cloud-functions`

## Rationale for local module implementation

Local modules are used where a purpose-built interface improves clarity for this application domain (for example, exposing messenger-specific outputs like function URI and secret IDs). This keeps composition simple in `infra/main.tf` while remaining compatible with CFF/TEF layering expectations.
