# Design: Implement encrypted messenger architecture on GCP

## Context

This change implements the architecture described in `ARCHITECTURE.md` using the
existing Terraform/IaC repository and OpenAPI contract in `openspec/specs/openapi.yaml`.
The target system is a privacy-first encrypted messenger with:

- GCP infrastructure managed by Terraform modules.
- A backend API on Cloud Functions (2nd gen).
- Firestore persistence.
- Secret Manager for JWT/VAPID secrets.
- Browser push notifications using standards-based Web Push + VAPID.
- Client-side cryptography in Rust/WASM (no plaintext on server).

## Goals

1. Build deployable IaC for core GCP services and least-privilege IAM.
2. Implement backend endpoints matching the OpenAPI contract.
3. Ensure storage schema and indexes support chat and key workflows.
4. Implement push subscription + delivery mechanics that never include plaintext.
5. Provide an implementation sequence that can be shipped incrementally.

## Non-goals

- Building a production-grade frontend UI.
- Full feature parity for post-PoC enhancements (group chats, media, backups).
- Over-optimizing performance before correctness and protocol safety checks.

## System Design

### Recommended Repository Structure

Use a monorepo layout that keeps app/runtime/infrastructure/pipeline concerns separated
but versioned together:

```text
repo-root/
├── apps/
│   └── pwa/                          # React + TypeScript PWA (Vite)
│       ├── src/
│       ├── public/
│       ├── tests/
│       ├── package.json
│       └── vite.config.ts
├── crates/
│   └── messenger-crypto/             # Rust crate compiled to WASM
│       ├── Cargo.toml
│       ├── src/
│       └── tests/
├── functions/                        # Cloud Functions API source
│   ├── src/
│   ├── tests/
│   └── package.json
├── infra/
│   ├── modules/                      # Reusable Terraform/Tofu modules
│   └── live/                         # Live env stacks (dev/stage/prod)
│       ├── dev/
│       ├── stage/
│       └── prod/
├── pipelines/                        # CI/CD and deployment pipeline definitions
│   ├── ci/
│   ├── cd/
│   └── scripts/
├── openspec/
└── docs/
```

Repository conventions:

- `apps/pwa` is the React app boundary; UI state and PWA service worker live here.
- `crates/messenger-crypto` is the only crypto implementation source of truth.
- `infra/modules` exposes reusable building blocks; `infra/live/*` composes them per environment.
- `pipelines/` contains reusable CI/CD definitions for lint/test/build/plan/deploy flows.

### 1) Infrastructure Layout (Terraform)

Create a root `infra/` module that composes:

- `modules/project-services`: enable required APIs.
- `modules/identity-platform`: configure Google login.
- `modules/firestore`: create Firestore DB and indexes.
- `modules/secret-manager`: create required secrets and IAM access.
- `modules/iam`: service account and role bindings.
- `modules/cloud-functions`: function deployment, source bucket/object, invoker IAM.

Foundation module usage guidance:

- Prefer `terraform-example-foundation` patterns/modules for security baseline, org/folder/project structure, and landing-zone guardrails where those layers are in scope.
- Prefer `cloud-foundation-fabric` modules/blueprints for workload-level components (APIs, IAM bindings, Firestore, Secret Manager, Cloud Run/Functions-adjacent resources) when they reduce bespoke Terraform and remain aligned with architecture constraints.
- If a required resource is not covered cleanly by either foundation toolkit, implement a local module with interfaces consistent with the selected foundation style.

Design choices:

- Keep module boundaries aligned with the architecture doc for easy ownership.
- Use explicit outputs between modules for cross-module wiring.
- Keep environment-specific values in `infra/environments/*.tfvars`.

### 2) Backend API Layout (Cloud Functions)

Implement endpoint handlers grouped by domain:

- `auth`: `POST /auth/google`
- `keys`: `POST /keys/identity`, `POST /keys/prekeys`, `GET /keys/bundle`
- `chat`: `POST /chat/send`, `GET /chat/history`, `GET /chat/poll`
- `push`: `POST /push/subscribe`, `DELETE /push/subscribe`

Design choices:

- Use a shared request middleware for app JWT parsing/validation.
- Allow unauthenticated ingress at Cloud Run layer, enforce auth in handler logic.
- Use structured error responses (`{ error: string }`) matching OpenAPI.

### 3) Data Model (Firestore)

Use collections aligned to architecture:

- `users/{uid}`
- `preKeyBundles/{uid}`
- `rooms/{roomId}`
- `rooms/{roomId}/messages/{messageId}`
- `pushSubscriptions/{uid}/devices/{deviceId}`

Indexes:

- Messages by `createdAt` ascending for per-room history pagination.
- Rooms by `participants` (array contains) and `lastMessageAt` descending for polling.

Design choices:

- Keep message payload opaque (`header`, `ciphertext`, optional X3DH bootstrap fields).
- Persist only public keys and prekey metadata server-side.
- Keep deterministic room ID generation (`SHA-256(sorted uids)`).

### 4) Security & Secrets

Secret Manager holds:

- `jwt-signing-key`
- `vapid-private-key`
- `vapid-public-key`

Design choices:

- Function runtime reads secrets at startup or cached lazy-load.
- JWT has short expiration and signer rotation path via secret versions.
- Push payload contains notification hint only (never message plaintext).

### 5) Crypto Integration Boundary

The repository tracks infrastructure and backend services. The Rust/WASM crypto module
is treated as an integration dependency exposed through API fields and documented data
contracts, with these boundaries:

- Server stores and forwards ciphertext and metadata only.
- Server validates structure and authorization, not cryptographic correctness.
- Client remains owner of key generation, ratchet state, and decrypt operations.

### 6) Pipeline Design

Pipeline stages should be modeled as reusable workflows:

1. `validate`: format and validate Terraform/Tofu, lint/typecheck app and functions, and run Rust tests.
2. `build`: build React PWA, compile WASM artifact, package functions.
3. `plan`: run Terraform/Tofu plan for `infra/live/<env>`.
4. `deploy`: deploy functions and static assets after gated approvals.

Design choices:

- Keep CI checks mandatory for every PR.
- Keep deploy jobs environment-scoped and approval-gated.
- Keep pipeline scripts in `pipelines/scripts/` so local and CI execution paths match.

## Execution Plan

1. Build Terraform module structure and root wiring.
2. Add Firestore indexes and IAM bindings required by backend handlers.
3. Implement auth and key-management endpoints first.
4. Implement chat send/history/poll endpoints with deterministic room handling.
5. Implement push subscribe/unsubscribe and notification fan-out on send.
6. Add smoke validation (format, validate, endpoint-level checks).

## Risks and Mitigations

- **Risk:** Missing Cloud credentials blocks `terraform plan`.
  - **Mitigation:** Treat `validate`/`fmt` as required CI checks; make `plan` optional in local.
- **Risk:** Prekey race conditions when consuming one-time keys.
  - **Mitigation:** Use transactional/atomic update patterns in Firestore.
- **Risk:** Auth mismatch between Identity Platform and app JWT.
  - **Mitigation:** Isolate auth flow and add endpoint-level integration tests.
- **Risk:** Push endpoint churn causes stale subscriptions.
  - **Mitigation:** Delete subscriptions on 410 Gone responses.

## Rollout Strategy

- Phase 1: Infrastructure and auth/key APIs.
- Phase 2: Chat and Firestore message flow.
- Phase 3: Push notifications and operational hardening.

Each phase should remain deployable independently with feature-gated usage on the client.
