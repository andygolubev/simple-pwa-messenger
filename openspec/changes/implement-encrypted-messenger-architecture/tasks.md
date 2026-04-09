- [x] 1. Bootstrap repository structure and shared Terraform foundations
  - [x] 1.0 Create the top-level monorepo structure with dedicated directories: `apps/pwa`, `crates/messenger-crypto`, `infra/live`, `infra/modules`, and `.github/workflows`.
  - [x] 1.1 Create root `infra/` files (`main.tf`, `variables.tf`, `outputs.tf`, `providers.tf`) and environment tfvars files for `dev` and `prod`.
  - [x] 1.2 Implement `modules/project-services` to enable required GCP APIs from the architecture and wire into root module.
  - [x] 1.3 Implement `modules/iam` to create Cloud Functions service account and baseline roles (`datastore.user`, `logging.logWriter`, `secretAccessor`).
  - [x] 1.4 Integrate `terraform-example-foundation` where applicable for secure org/folder/project baseline and document reused vs custom layers.
  - [x] 1.5 Run `terraform fmt -check` and `terraform validate` in `infra/`.

- [x] 2. Implement infrastructure modules for runtime services
  - [x] 2.1 Implement `modules/identity-platform` for Identity Platform config and Google provider setup.
  - [x] 2.2 Implement `modules/firestore` for default Firestore database and required indexes for room/message queries.
  - [x] 2.3 Implement `modules/secret-manager` for VAPID and JWT secrets plus IAM access for function service account.
  - [x] 2.4 Implement `modules/cloud-functions` with function source bucket/object, one deployed function service, and invoker IAM for public HTTPS.
  - [x] 2.5 Reuse `cloud-foundation-fabric` modules/blueprints where applicable for service-level resources, preferring composition over bespoke resources.
  - [x] 2.6 Wire all modules in root `infra/main.tf` and expose function URLs/secrets references through outputs.
  - [x] 2.7 Run `terraform fmt -check` and `terraform validate` in `infra/`.

- [ ] 3. Deliver Cloud Functions API implementation
  - [ ] 3.1 Scaffold `functions/` with runtime (`package.json` + TypeScript tooling or Python equivalent) and shared auth/middleware utilities.
  - [ ] 3.2 Implement `POST /auth/google` to verify Google ID token, upsert `users/{uid}`, and mint app JWT.
  - [ ] 3.3 Implement key endpoints: `POST /keys/identity`, `POST /keys/prekeys`, `GET /keys/bundle` including one-time prekey pop behavior.
  - [ ] 3.4 Implement chat endpoints: `POST /chat/send`, `GET /chat/history`, `GET /chat/poll` storing encrypted payload fields only.
  - [ ] 3.5 Implement push endpoints: `POST /push/subscribe` and `DELETE /push/subscribe`, and push fan-out in `chat/send` with stale subscription cleanup.
  - [ ] 3.6 Add endpoint-level tests (unit/integration with mocks) for auth, key bundle lifecycle, chat write/read, and push subscription flows.
  - [ ] 3.7 Run function test command and lint/typecheck.

- [ ] 4. Deliver Rust/WASM cryptography crate and bridge contracts
  - [ ] 4.1 Create `crates/messenger-crypto` with module layout from architecture (`identity.rs`, `x3dh.rs`, `double_ratchet.rs`, `session.rs`, etc.).
  - [ ] 4.2 Implement wasm-bindgen exports for identity generation, prekey generation, X3DH initiate/respond, and ratchet encrypt/decrypt.
  - [ ] 4.3 Enforce pure Rust crypto dependencies with browser CSPRNG support (`getrandom/js`) and zeroization of sensitive key material.
  - [ ] 4.4 Add Rust tests for X3DH agreement equivalence, ratchet progression, and out-of-order message handling.
  - [ ] 4.5 Add a build script for wasm-pack output to `pwa/pkg` and verify the crate compiles for `wasm32-unknown-unknown`.

- [ ] 5. Deliver PWA foundation and encrypted chat client flow
  - [ ] 5.1 Scaffold a React PWA in `apps/pwa/` with service worker, manifest, API client, auth flow, and IndexedDB persistence layer for keys/sessions.
  - [ ] 5.2 Implement login flow (`Sign in with Google` -> `/auth/google`) and first-login key bootstrap (`generate_identity`, signed prekey, one-time prekeys).
  - [ ] 5.3 Implement key publication and replenishment checks (`countOnly=true`, rotate signed prekey every 30 days, replenish OPKs below threshold).
  - [ ] 5.4 Implement 1:1 room derivation and message send/receive pipeline using X3DH for first message and Double Ratchet for subsequent messages.
  - [ ] 5.5 Implement Web Push subscribe/unsubscribe and service worker push/click handlers with payload privacy guarantees.
  - [ ] 5.6 Add client tests for IndexedDB persistence and API contract adapters; run frontend test/lint/build pipeline.

- [ ] 6. Operational validation, documentation, and hardening
  - [ ] 6.1 Ensure `openspec/specs/openapi.yaml` reflects implemented request/response contracts if runtime choices diverge.
  - [ ] 6.2 Add runbooks for provisioning (`terraform init/plan/apply`), function deployment, and local dev commands for functions + PWA + WASM build.
  - [ ] 6.3 Add security checklist for JWT rotation, secret access scope, TOFU identity warnings, and push payload plaintext prohibition.
  - [ ] 6.4 Execute end-to-end validation: auth, first encrypted message, subsequent ratcheted messages, and push notification receipt.
  - [ ] 6.5 Capture post-deployment verification artifacts (logs/screenshots) and update OpenSpec task statuses when complete.
  - [ ] 6.6 Implement CI/CD pipelines under `pipelines/` (and bridge to runner-specific entrypoints like `.github/workflows` when needed) for Terraform/Tofu checks (`fmt`, `validate`, optional `plan`), Rust/WASM build+tests, and PWA build+tests.
