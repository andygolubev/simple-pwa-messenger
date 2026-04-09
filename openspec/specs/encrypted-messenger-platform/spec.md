## Requirements

### Requirement: Google sign-in bootstraps application identity
The platform MUST authenticate users with Google sign-in and mint an application JWT used for all authenticated API operations.

#### Scenario: Successful Google sign-in
- **WHEN** a client submits a valid Google ID token to `POST /auth/google`
- **THEN** the backend verifies the token with Google Identity Platform
- **AND** upserts `users/{uid}` with profile metadata
- **AND** returns an app JWT containing the authenticated `uid`
- **AND** includes `isNewUser` indicating whether this is first-time provisioning.

#### Scenario: Invalid sign-in attempt
- **WHEN** `POST /auth/google` receives an invalid, expired, or unverifiable Google ID token
- **THEN** the backend rejects the request
- **AND** does not mint an app JWT.

### Requirement: Client devices own private key material
The platform MUST generate and retain identity private keys only on client devices and MUST NOT store private key material on backend systems.

#### Scenario: First login key bootstrap
- **WHEN** `isNewUser=true` is returned after authentication
- **THEN** the client generates identity and signing keypairs locally in the Rust-to-WASM crypto module
- **AND** stores private keys in client-side storage
- **AND** uploads only public key material to backend key endpoints.

#### Scenario: Backend key storage boundaries
- **WHEN** key registration APIs persist key data
- **THEN** only public identity/signing keys, signed prekeys, and one-time prekey public keys are stored server-side
- **AND** private keys are never accepted or persisted by backend services.

### Requirement: Session establishment uses X3DH-compatible prekey bundles
The platform MUST support initial encrypted session establishment through identity keys, signed prekeys, and optional one-time prekeys.

#### Scenario: Fetch recipient prekey bundle
- **WHEN** an authenticated sender calls `GET /keys/bundle?uid={uid}`
- **THEN** the backend returns recipient identity public key, signing public key, signed prekey, and signature
- **AND** returns one one-time prekey when available.

#### Scenario: First encrypted message consumes one-time prekey
- **WHEN** sender submits an initial encrypted envelope with `type=x3dh_initial` and `usedOneTimePreKeyId`
- **THEN** backend stores the encrypted envelope
- **AND** atomically marks the referenced recipient one-time prekey as consumed
- **AND** updates one-time prekey availability metadata.

### Requirement: Ongoing message encryption follows Double Ratchet state evolution
The platform MUST store and transport encrypted Double Ratchet envelopes and allow clients to advance ratchet state per message.

#### Scenario: Store encrypted envelope
- **WHEN** an authenticated sender calls `POST /chat/send`
- **THEN** the backend persists only encrypted envelope fields such as `header`, `ciphertext`, and session metadata
- **AND** does not persist plaintext message content.

#### Scenario: Retrieve encrypted history
- **WHEN** an authenticated client calls `GET /chat/history`
- **THEN** backend returns encrypted message envelopes ordered by server timestamp
- **AND** supports pagination parameters for incremental history loading.

### Requirement: Firestore persists encrypted artifacts and platform metadata
The platform MUST persist data in Firestore collections that separate users, prekey bundles, rooms, encrypted messages, and push subscriptions.

#### Scenario: Room and message persistence model
- **WHEN** a 1:1 chat is created or used
- **THEN** a deterministic `roomId` derived from participant UIDs identifies the room
- **AND** encrypted messages are stored under `rooms/{roomId}/messages/{messageId}` with sender and timestamp metadata.

#### Scenario: Key bundle persistence model
- **WHEN** `/keys/identity` and `/keys/prekeys` requests succeed
- **THEN** backend persists identity/signing public keys, signed prekey state, one-time prekeys, and one-time prekey count metadata.

### Requirement: Authenticated API surface protects key, chat, and push endpoints
The platform MUST expose backend endpoints for key management, chat, and push operations and require valid app JWTs for all non-auth endpoints.

#### Scenario: Access protected endpoint with valid token
- **WHEN** a client calls `/keys/*`, `/chat/*`, or `/push/*` with a valid app JWT
- **THEN** backend authorizes the request and executes endpoint logic.

#### Scenario: Access protected endpoint without valid token
- **WHEN** app JWT is missing, invalid, or expired on `/keys/*`, `/chat/*`, or `/push/*`
- **THEN** backend rejects the request as unauthorized.

### Requirement: Web Push notifications use VAPID with privacy-preserving payloads
The platform MUST manage push subscriptions and send message notifications without exposing plaintext message bodies.

#### Scenario: Register push subscription
- **WHEN** an authenticated client calls `POST /push/subscribe` with a valid Web Push subscription
- **THEN** backend stores subscription data at `pushSubscriptions/{uid}/devices/{deviceId}`.

#### Scenario: Send notification hint on new message
- **WHEN** an encrypted message is accepted by `POST /chat/send`
- **THEN** backend sends Web Push notifications using VAPID credentials
- **AND** payloads include only notification hint metadata (for example title, body hint, and `roomId`)
- **AND** payloads never include plaintext message content.

### Requirement: Secret Manager provides runtime secrets to backend services
The platform MUST source sensitive server-side values from Secret Manager rather than hardcoded configuration.

#### Scenario: Runtime secret access
- **WHEN** backend functions initialize runtime integrations
- **THEN** VAPID keys and JWT signing keys are read from Secret Manager
- **AND** function service accounts require only least-privilege secret access.

### Requirement: Infrastructure is provisioned via Terraform module composition
The platform MUST be reproducibly provisioned from Terraform/OpenTofu modules for required GCP services.

#### Scenario: Baseline platform provisioning
- **WHEN** infrastructure is planned or applied
- **THEN** module composition provisions Identity Platform, Cloud Functions (2nd gen), Firestore, Secret Manager, IAM bindings, and required APIs
- **AND** environment inputs include at least `project_id` and `region`.

#### Scenario: Required APIs are enabled
- **WHEN** baseline platform infrastructure is provisioned
- **THEN** the project enables required APIs for functions, run, firestore, secret manager, identity toolkit, build, artifact registry, logging, and IAM.
