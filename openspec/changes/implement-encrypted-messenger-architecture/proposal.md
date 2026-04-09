## Summary

Implement the encrypted messenger architecture defined in `ARCHITECTURE.md` by delivering:
- Terraform modules for GCP infrastructure (Identity Platform, Cloud Functions 2nd gen, Firestore, Secret Manager, IAM, and required APIs).
- Backend API endpoints for auth, key management, encrypted messaging, and push subscriptions.
- A PWA shell that integrates with a Rust-to-WASM crypto crate implementing Signal protocol primitives (X3DH + Double Ratchet).
- Operational/security guardrails for key material handling, JWT issuance, and push payload privacy.

## Problem

The repository currently contains architecture intent and an OpenAPI contract, but no implementation plan broken into executable features and milestones. Without a concrete OpenSpec change, implementation can drift from the architecture and leave gaps in encryption guarantees, infra provisioning, and API behavior.

## Goals

1. Translate the architecture into a traceable, implementation-ready feature plan.
2. Ensure all required backend/platform capabilities are explicitly captured as requirements.
3. Define phased tasks that can be applied incrementally with verification at each step.
4. Keep the implementation aligned with low-cost GCP primitives and E2E-encryption constraints.

## Non-goals

1. Implementing group chat, multi-device sync, media/file encryption, or key backup flows.
2. Replacing the architecture decisions with alternate protocol stacks.
3. Running production `terraform apply` against real GCP environments within this change.

## Scope

### In scope

- Infra module implementation plan (`infra/` + `infra/modules/*`) and wiring.
- Cloud Functions API implementation plan for the endpoints in `openspec/specs/openapi.yaml`.
- PWA + WASM integration plan for identity/prekey/session lifecycle and local secure storage.
- Firestore schema, indexes, and message flow constraints required by the architecture.
- Web Push subscription and delivery flow based on VAPID.
- Recommended repository structure for app/runtime/infra and CI/CD ownership boundaries.

### Out of scope

- Post-PoC enhancements listed in `ARCHITECTURE.md` future section.
- Non-GCP deployments.

## Success Criteria

1. OpenSpec artifacts exist and define feature requirements and implementation tasks for architecture delivery.
2. Tasks are ordered to allow iterative delivery: infra baseline -> auth/keys -> chat/push -> PWA/WASM integration -> hardening.
3. Requirements explicitly preserve end-to-end encryption constraints (server never processes plaintext).

## Risks and Mitigations

- **Risk:** Protocol/crypto complexity introduces implementation drift.  
  **Mitigation:** Separate WASM crypto module tasks with dedicated test milestones.
- **Risk:** Firestore consistency issues for one-time prekey consumption.  
  **Mitigation:** Require transactional prekey pop semantics and explicit task coverage.
- **Risk:** Push system leaks message content.  
  **Mitigation:** Requirement that push payload only contains notification hints, never plaintext.
