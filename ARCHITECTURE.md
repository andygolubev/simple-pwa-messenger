# Encrypted Messenger — GCP Architecture

## Overview

A privacy-first PWA messenger where users log in with Google, generate a client-side keypair (like blockchain wallets), encrypt every message end-to-end using a **Rust-to-WASM crypto module**, and receive standards-based Web Push notifications — all running on the cheapest GCP primitives.

---

## Design Principles

| Principle | Detail |
|---|---|
| **E2E encryption** | Messages are encrypted on the sender's device with the receiver's public key. The server never sees plaintext. |
| **Rust WASM crypto** | All cryptographic operations run in a Rust module compiled to WebAssembly — no JS crypto, no subtle API. |
| **Signal Protocol** | X3DH key agreement + Double Ratchet for forward secrecy and future secrecy per message. |
| **Key-based identity** | Each user generates a Curve25519 identity keypair on first login. The private key never leaves the device. |
| **Cheapest GCP stack** | Cloud Functions + Firestore + Secret Manager. No GKE, no Pub/Sub, no FCM. |
| **Standards-based push** | W3C Push API + VAPID. Works on Android and iOS (Home Screen PWA). |
| **Terraform/Tofu ready** | Every GCP resource is designed to be provisioned as a module. |

---

## Crypto Approach: Signal Protocol in Pure Rust → WASM

The official `libsignal-client` Rust crate links against BoringSSL (C library with platform-specific assembly) and **does not compile to `wasm32-unknown-unknown`**.

**Solution:** Implement the Signal Protocol (X3DH + Double Ratchet) in pure Rust using the RustCrypto / dalek ecosystem. These crates compile cleanly to WASM. The protocol logic lives in our own crate (`messenger-crypto`), exposed to the PWA via `wasm-bindgen`.

---

## Rust WASM Crypto Module

### Crate: `messenger-crypto`

```
pwa/
├── crates/
│   └── messenger-crypto/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs              # wasm_bindgen entry points
│       │   ├── identity.rs         # Identity keypair generation & management
│       │   ├── x3dh.rs             # Extended Triple Diffie-Hellman key agreement
│       │   ├── double_ratchet.rs   # Double Ratchet session state machine
│       │   ├── session.rs          # Session management (create, encrypt, decrypt)
│       │   ├── keys.rs             # Key types: IdentityKey, SignedPreKey, OneTimePreKey
│       │   ├── store.rs            # Trait for persistent storage (IndexedDB bridge)
│       │   └── error.rs            # Error types
│       └── tests/
│           ├── x3dh_test.rs
│           ├── ratchet_test.rs
│           └── integration_test.rs
│
├── pkg/                            # wasm-pack output (gitignored, built in CI)
│   ├── messenger_crypto_bg.wasm
│   ├── messenger_crypto.js         # JS glue code
│   └── messenger_crypto.d.ts       # TypeScript types
│
├── src/                            # PWA frontend (TypeScript)
│   ├── crypto-bridge.ts            # Thin wrapper: calls WASM, manages IndexedDB storage
│   └── ...
│
└── build.sh                        # wasm-pack build --target web --release
```

### Rust Dependencies

```toml
# crates/messenger-crypto/Cargo.toml
[package]
name = "messenger-crypto"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]     # cdylib for WASM, rlib for native tests

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
js-sys = "0.3"

# Crypto (all pure-Rust, all compile to wasm32-unknown-unknown)
x25519-dalek = { version = "2", features = ["static_secrets"] }
ed25519-dalek = { version = "2", features = ["rand_core"] }
curve25519-dalek = "4"
aes-gcm = "0.10"                    # AES-256-GCM for message encryption
hkdf = "0.12"
sha2 = "0.10"
hmac = "0.12"
rand_core = { version = "0.6", features = ["getrandom"] }
getrandom = { version = "0.2", features = ["js"] }  # CRITICAL: "js" feature for browser CSPRNG
base64 = "0.22"
zeroize = { version = "1", features = ["derive"] }   # Secure memory cleanup

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "z"          # Optimize for binary size
lto = true               # Link-time optimization
codegen-units = 1        # Single codegen unit for best optimization
strip = true             # Strip debug symbols
```

### WASM Build

```bash
# Build command (used in CI and local dev)
wasm-pack build crates/messenger-crypto \
  --target web \
  --release \
  --out-dir ../../pkg

# Expected output size: ~150-250 KB gzipped
```

### Exported WASM API (`#[wasm_bindgen]`)

```rust
// === Identity ===

#[wasm_bindgen]
pub fn generate_identity() -> JsValue;
// Returns: { identityKeyPair: { public: base64, private: base64 },
//            signingKeyPair:  { public: base64, private: base64 } }
// Called once on first login. Private keys → IndexedDB. Public keys → server.

#[wasm_bindgen]
pub fn generate_signed_prekey(identity_private: &str, prekey_id: u32) -> JsValue;
// Returns: { id, publicKey, signature, privateKey }
// Signed with identity key. Rotate monthly.

#[wasm_bindgen]
pub fn generate_one_time_prekeys(start_id: u32, count: u32) -> JsValue;
// Returns: [{ id, publicKey, privateKey }, ...]
// Batch generate. Upload public parts to server. Store private parts in IndexedDB.

// === X3DH (Session Establishment) ===

#[wasm_bindgen]
pub fn x3dh_initiate(
    my_identity_private: &str,
    their_identity_public: &str,
    their_signed_prekey: &str,
    their_signed_prekey_signature: &str,
    their_one_time_prekey: Option<String>,  // may be absent if server has none left
) -> JsValue;
// Returns: { sessionState: base64, ephemeralPublic: base64, usedOneTimePreKeyId: Option<u32> }
// Sender calls this to create a new session with a recipient.
// Verifies signed prekey signature before proceeding.

#[wasm_bindgen]
pub fn x3dh_respond(
    my_identity_private: &str,
    my_signed_prekey_private: &str,
    my_one_time_prekey_private: Option<String>,
    their_identity_public: &str,
    their_ephemeral_public: &str,
) -> JsValue;
// Returns: { sessionState: base64 }
// Receiver calls this when they get the first message from a new sender.

// === Double Ratchet (Message Encrypt/Decrypt) ===

#[wasm_bindgen]
pub fn ratchet_encrypt(
    session_state: &str,   // base64 serialized session
    plaintext: &str,
) -> JsValue;
// Returns: { updatedSessionState: base64, message: { header: base64, ciphertext: base64 } }
// Advances the sending ratchet. Caller must persist updatedSessionState.

#[wasm_bindgen]
pub fn ratchet_decrypt(
    session_state: &str,
    header: &str,
    ciphertext: &str,
) -> JsValue;
// Returns: { updatedSessionState: base64, plaintext: string }
// Advances the receiving ratchet. Handles out-of-order messages via skipped message keys.

// === Utilities ===

#[wasm_bindgen]
pub fn verify_identity_signature(
    signing_public: &str,
    message: &str,
    signature: &str,
) -> bool;
// Verify Ed25519 signature. Used for signed prekey verification and message authentication.
```

### IndexedDB Storage (Client-Side)

The WASM module is stateless — all key material and session state is passed in/out as base64 strings. The TypeScript bridge layer persists them in IndexedDB:

```
IndexedDB database: "messenger-crypto"

Object stores:
├── identity/
│   ├── identityPrivateKey: base64        # X25519 private key
│   ├── identityPublicKey: base64         # X25519 public key
│   ├── signingPrivateKey: base64         # Ed25519 private key
│   └── signingPublicKey: base64          # Ed25519 public key
│
├── signedPreKeys/{id}
│   ├── id: u32
│   ├── privateKey: base64
│   ├── publicKey: base64
│   └── createdAt: timestamp
│
├── oneTimePreKeys/{id}
│   ├── id: u32
│   └── privateKey: base64                # Public key already uploaded to server
│
├── sessions/{recipientUid}
│   └── state: base64                     # Serialized Double Ratchet session state
│
└── knownIdentities/{uid}
    └── identityPublicKey: base64         # Trust-on-first-use (TOFU) identity verification
```

---

## High-Level Diagram

```
┌───────────────────────────────────────────────────────────────────────────┐
│                              PWA (Browser)                                │
│                                                                           │
│  ┌──────────────┐  ┌──────────────────────┐  ┌────────────────────────┐  │
│  │ Google Sign-In│  │ messenger-crypto.wasm│  │ Service Worker         │  │
│  │ (Identity     │  │                      │  │ (Web Push receive,     │  │
│  │  Platform JS) │  │ • generate_identity  │  │  offline cache)        │  │
│  │              │  │ • x3dh_initiate     │  │                        │  │
│  │              │  │ • x3dh_respond      │  │                        │  │
│  │              │  │ • ratchet_encrypt   │  │                        │  │
│  │              │  │ • ratchet_decrypt   │  │                        │  │
│  │              │  │ • generate_prekeys  │  │                        │  │
│  └──────┬───────┘  └──────────┬───────────┘  └───────────┬────────────┘  │
│         │                     │                          │                │
│         │              ┌──────┴───────┐                  │                │
│         │              │  IndexedDB   │                  │                │
│         │              │ (keys,       │                  │                │
│         │              │  sessions)   │                  │                │
│         │              └──────────────┘                  │                │
└─────────┼─────────────────────┼──────────────────────────┼────────────────┘
          │                     │                          │
          │  HTTPS              │  HTTPS                   │  Web Push (VAPID)
          ▼                     ▼                          ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                       Cloud Functions (2nd gen)                            │
│                                                                           │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐│
│  │ /auth/google  │ │ /keys/       │ │ /chat/send   │ │ /push/subscribe  ││
│  │              │ │  bundle      │ │ /chat/history│ │                  ││
│  │              │ │  prekeys     │ │ /chat/poll   │ │                  ││
│  │              │ │  identity    │ │              │ │                  ││
│  └──────┬───────┘ └──────┬───────┘ └──────┬───────┘ └────────┬─────────┘│
│         │                │                │                  │           │
└─────────┼────────────────┼────────────────┼──────────────────┼───────────┘
          │                │                │                  │
          ▼                ▼                ▼                  ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                             Firestore                                     │
│                                                                           │
│ users/ preKeyBundles/ rooms/ rooms/{id}/messages/ pushSubscriptions/      │
└───────────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌──────────────────┐       ┌──────────────────┐
│  Secret Manager  │       │  Cloud Logging   │
│  (VAPID keys,    │       │  (built-in)      │
│   JWT secret)    │       │                  │
└──────────────────┘       └──────────────────┘
```

---

## GCP Services

### 1. Cloud Identity Platform

**Purpose:** Google Account login (the only auth provider for PoC).

| Setting | Value |
|---|---|
| Provider | Google |
| Multi-tenancy | Disabled (single tenant) |
| MFA | Disabled for PoC |

**Terraform module:** `modules/identity-platform`

Resources:
- `google_identity_platform_config`
- `google_identity_platform_default_supported_idp_config` (Google provider)

**Flow:**
1. PWA loads Identity Platform JS SDK.
2. User clicks "Sign in with Google".
3. SDK returns a Google ID token.
4. PWA sends the token to `POST /auth/google`.
5. Cloud Function verifies the token via Identity Platform Admin SDK, creates/updates user doc, returns an app JWT.

---

### 2. Cloud Functions (2nd gen)

**Purpose:** All backend API endpoints. Single function per endpoint (or one function with routing — your choice at implementation).

| Setting | Value |
|---|---|
| Runtime | Node.js 20 (or Python 3.12) |
| Gen | 2nd gen (Cloud Run-backed) |
| Memory | 256 MB (minimum) |
| Min instances | 0 (scale to zero) |
| Max instances | 5 (PoC cap) |
| Timeout | 60 s |
| Ingress | Allow all (HTTPS) |
| Auth | `--allow-unauthenticated` (app JWT checked in code) |

**Terraform module:** `modules/cloud-functions`

Resources:
- `google_cloudfunctions2_function` (one per endpoint or one router function)
- `google_cloud_run_service_iam_member` (allUsers invoker for HTTPS)
- `google_storage_bucket` + `google_storage_bucket_object` (function source zip)
- `google_service_account` (dedicated SA for functions)

#### Endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/auth/google` | None (public) | Verify Google ID token, upsert user, return app JWT |
| POST | `/keys/identity` | JWT | Publish user's identity public key + signing public key |
| POST | `/keys/prekeys` | JWT | Upload signed prekey + batch of one-time prekeys |
| GET | `/keys/bundle?uid={uid}` | JWT | Fetch a prekey bundle for starting a session with a user |
| POST | `/chat/send` | JWT | Store an encrypted message (Double Ratchet output) |
| GET | `/chat/history?roomId={id}&limit=50&after={cursor}` | JWT | Return encrypted messages (paginated) |
| GET | `/chat/poll?since={timestamp}` | JWT | Return rooms with new messages since timestamp |
| POST | `/push/subscribe` | JWT | Store Web Push subscription for the user/device |
| DELETE | `/push/subscribe` | JWT | Remove a subscription (logout / unsubscribe) |

---

### 3. Firestore (Native mode)

**Purpose:** All persistent data.

| Setting | Value |
|---|---|
| Mode | Native |
| Location | `europe-west1` (or your region) |
| Billing | Free tier: 1 GiB storage, 50K reads/day, 20K writes/day |

**Terraform module:** `modules/firestore`

Resources:
- `google_firestore_database` (default database)
- `google_firestore_index` (composite indexes as needed)

#### Collections & Documents

```
users/{uid}
├── displayName: string
├── email: string
├── photoURL: string
├── createdAt: timestamp
└── lastLoginAt: timestamp

preKeyBundles/{uid}
├── identityPublicKey: string            // base64 X25519 identity public key
├── signingPublicKey: string             // base64 Ed25519 signing public key
├── signedPreKey:
│   ├── id: number
│   ├── publicKey: string                // base64
│   └── signature: string               // base64 Ed25519 signature over publicKey
├── oneTimePreKeys: [                    // Array — server pops one per new session
│   { id: number, publicKey: string },
│   ...
│ ]
├── signedPreKeyUpdatedAt: timestamp
└── oneTimePreKeyCount: number           // Denormalized count for replenishment check

rooms/{roomId}
├── participants: string[]               // [uid1, uid2] — sorted for deterministic ID
├── createdAt: timestamp
└── lastMessageAt: timestamp

rooms/{roomId}/messages/{messageId}
├── senderUid: string
├── type: "x3dh_initial" | "ratchet"     // First message in session vs subsequent
├── header: string                       // base64 Double Ratchet header
│                                        //   (ratchet public key, previous chain length, message number)
├── ciphertext: string                   // base64 AES-256-GCM encrypted payload
├── ephemeralPublic: string | null       // base64 — present only on type "x3dh_initial"
├── usedOneTimePreKeyId: number | null   // present only on type "x3dh_initial"
├── createdAt: timestamp
└── serverTimestamp: timestamp           // Server-set for ordering

pushSubscriptions/{uid}/devices/{deviceId}
├── endpoint: string                     // Web Push endpoint URL
├── keys.p256dh: string                  // client public key
├── keys.auth: string                    // client auth secret
├── createdAt: timestamp
└── userAgent: string                    // for debugging
```

**Room ID convention:** For 1:1 chats, derive roomId deterministically:
`roomId = SHA-256(sort([uid1, uid2]).join(":"))` — so both users resolve to the same room.

#### Indexes

| Collection | Fields | Order |
|---|---|---|
| `rooms/{roomId}/messages` | `createdAt` | ASC |
| `rooms` | `participants` (Array Contains) + `lastMessageAt` | DESC |

---

### 4. Secret Manager

**Purpose:** Store server-side secrets that Cloud Functions need at runtime.

| Setting | Value |
|---|---|
| Billing | Free tier: 6 active secret versions, 10K access ops/month |

**Terraform module:** `modules/secret-manager`

Resources:
- `google_secret_manager_secret` (one per secret)
- `google_secret_manager_secret_version` (actual value)
- `google_secret_manager_secret_iam_member` (grant CF service account `secretAccessor`)

#### Secrets

| Secret name | Content |
|---|---|
| `vapid-private-key` | VAPID private key (ECDSA P-256) for Web Push signing |
| `vapid-public-key` | VAPID public key (shared with frontend, but stored here for single source of truth) |
| `jwt-signing-key` | HMAC secret (or RSA private key) for minting app JWTs |

---

### 5. Cloud Logging (built-in)

**Purpose:** Function logs, errors, structured request tracing.

No extra Terraform needed — Cloud Functions 2nd gen logs automatically to Cloud Logging.

Optional: create a `google_logging_project_sink` if you want to export logs to a bucket for long-term storage.

---

## Signal Protocol — Encryption Architecture

### Why Signal Protocol

| Property | Simple keypair (v1 architecture) | Signal Protocol (X3DH + Double Ratchet) |
|---|---|---|
| Forward secrecy | None — compromise of long-term key exposes all past messages | Per-message — each message uses a unique key derived from ephemeral ratchet state |
| Future secrecy | None | Yes — compromise of current state self-heals after a round-trip |
| Key compromise impact | Catastrophic — all history readable | Limited — only messages in current ratchet step |
| Offline first-message | Yes | Yes — via prekey bundles stored on server |
| Replay protection | Manual (nonce + timestamp) | Built-in (message counters in ratchet) |
| Industry adoption | None | Signal, WhatsApp, Facebook Messenger, Google Messages |

### Protocol Components

```
Signal Protocol = X3DH (session establishment) + Double Ratchet (message encryption)

Key types:
┌─────────────────────────────────────────────────────────────────────┐
│ Identity Key (IK)        Long-term X25519 keypair.                  │
│                          Generated once. Public part on server.     │
│                          Private part in IndexedDB forever.         │
│                                                                     │
│ Signing Key (SK)         Long-term Ed25519 keypair.                 │
│                          Signs prekeys. Verifies identity.          │
│                                                                     │
│ Signed Pre-Key (SPK)     Medium-term X25519 keypair.                │
│                          Rotated monthly. Signed by identity key.   │
│                          Public part + signature on server.         │
│                                                                     │
│ One-Time Pre-Key (OPK)   Ephemeral X25519 keypair.                  │
│                          Used once, then deleted. Server pops one   │
│                          per new session. Batch-uploaded.            │
│                                                                     │
│ Ephemeral Key (EK)       Per-session X25519 keypair.                 │
│                          Generated by sender during X3DH.           │
│                          Sent in first message, then discarded.     │
│                                                                     │
│ Ratchet Key (RK)         Per-message X25519 keypair.                 │
│                          Rotated every time the sender changes.     │
│                          Provides forward secrecy.                  │
└─────────────────────────────────────────────────────────────────────┘
```

### X3DH: Session Establishment

```
Alice wants to message Bob for the first time.

1. Alice fetches Bob's prekey bundle from server:
   GET /keys/bundle?uid=bob

   Response:
   {
     identityPublicKey:       IK_B,
     signingPublicKey:        SK_B,     // Ed25519 verify key
     signedPreKey:            SPK_B,    // { id, publicKey, signature }
     oneTimePreKey:           OPK_B     // { id, publicKey } — or null if exhausted
   }

2. Alice's WASM module (x3dh_initiate):
   a. Verify SPK_B.signature using SK_B           // Reject if invalid
   b. Generate ephemeral keypair EK_A
   c. Compute 3 (or 4) DH values:
      DH1 = X25519(IK_A_private,  SPK_B)          // Identity ↔ SignedPreKey
      DH2 = X25519(EK_A_private,  IK_B)           // Ephemeral ↔ Identity
      DH3 = X25519(EK_A_private,  SPK_B)          // Ephemeral ↔ SignedPreKey
      DH4 = X25519(EK_A_private,  OPK_B)          // Ephemeral ↔ OneTimePreKey (if available)
   d. master_secret = HKDF(DH1 || DH2 || DH3 [|| DH4])
   e. Initialize Double Ratchet with master_secret → session state

3. Alice sends first message to server:
   POST /chat/send {
     roomId,
     type: "x3dh_initial",
     ephemeralPublic: EK_A.public,
     usedOneTimePreKeyId: OPK_B.id,               // so Bob knows which OPK to use
     header: <ratchet header>,
     ciphertext: <ratchet encrypted message>
   }

4. Server:
   a. Store message in Firestore
   b. Delete used OPK from Bob's prekey bundle (atomic pop)
   c. If Bob's OPK count < threshold → flag for replenishment
   d. Send Web Push to Bob

5. Bob receives and processes (x3dh_respond):
   a. Look up OPK private key by usedOneTimePreKeyId from IndexedDB
   b. Compute same DH values using his private keys + Alice's public keys
   c. Derive same master_secret
   d. Initialize Double Ratchet → session state
   e. Decrypt first message using ratchet_decrypt
   f. Delete used OPK from IndexedDB (one-time use)
```

### Double Ratchet: Ongoing Messages

```
After X3DH establishes the session, all subsequent messages use the Double Ratchet:

┌─────────────┐                              ┌─────────────┐
│   Alice      │                              │     Bob      │
│              │                              │              │
│  ratchet     │   { header, ciphertext }     │  ratchet     │
│  _encrypt()  ├─────────────────────────────►│  _decrypt()  │
│              │                              │              │
│  ratchet     │   { header, ciphertext }     │  ratchet     │
│  _decrypt()  │◄─────────────────────────────┤  _encrypt()  │
│              │                              │              │
└─────────────┘                              └─────────────┘

Each direction change triggers a DH ratchet step (new ephemeral keypair).
Each message advances a symmetric chain ratchet (HMAC-based KDF).

Header contains:
- Sender's current ratchet public key
- Previous sending chain message count
- Current message number

The receiver uses the header to:
1. Detect if a new DH ratchet step is needed
2. Derive the correct message key
3. Handle out-of-order messages (up to a configurable window)

Encryption per message: AES-256-GCM(message_key, plaintext)
```

### Prekey Replenishment

```
Client checks prekey health on each app open:

1. GET /keys/bundle?uid={myUid}&countOnly=true
   → { oneTimePreKeyCount: N }

2. If N < LOW_THRESHOLD (e.g., 5):
   a. Generate batch of new OPKs in WASM: generate_one_time_prekeys(nextId, 20)
   b. Store private keys in IndexedDB
   c. POST /keys/prekeys { oneTimePreKeys: [{ id, publicKey }, ...] }

3. Signed prekey rotation:
   - If signedPreKey age > 30 days:
     a. Generate new SPK in WASM: generate_signed_prekey(identityPrivate, newId)
     b. POST /keys/prekeys { signedPreKey: { id, publicKey, signature } }
     c. Keep old SPK in IndexedDB for a grace period (messages in flight may reference it)
```

---

## Web Push Architecture

### VAPID Setup

```
Generate once (store in Secret Manager):
  - VAPID private key (ECDSA P-256)
  - VAPID public key  (shared with frontend)
  - VAPID subject: "mailto:admin@yourdomain.com"
```

### Subscription Flow

```
PWA:
1. Register Service Worker
2. Request Notification.permission
3. serviceWorkerRegistration.pushManager.subscribe({
     userVisibleOnly: true,
     applicationServerKey: VAPID_PUBLIC_KEY
   })
4. POST /push/subscribe { subscription JSON }

Cloud Function:
1. Store subscription in pushSubscriptions/{uid}/devices/{deviceId}
```

### Notification Flow

```
When POST /chat/send is called:
1. Store message in Firestore
2. Load recipient's push subscriptions from pushSubscriptions/{recipientUid}/devices/*
3. For each subscription:
   a. Build Web Push payload: { title: "New message", body: "You have a new encrypted message", roomId }
      (payload must NOT contain plaintext — only a notification hint)
   b. Send via web-push library (Node.js) or pywebpush (Python) using VAPID keys
   c. If endpoint returns 410 Gone → delete that subscription from Firestore
```

### Service Worker Handler

```javascript
self.addEventListener('push', event => {
  const data = event.data.json();
  self.registration.showNotification(data.title, {
    body: data.body,
    data: { roomId: data.roomId }
  });
});

self.addEventListener('notificationclick', event => {
  event.notification.close();
  clients.openWindow(`/chat/${event.notification.data.roomId}`);
});
```

---

## Authentication Flow

```
┌──────────┐         ┌──────────────────┐         ┌──────────────────┐
│   PWA    │         │ Identity Platform│         │ Cloud Function   │
│          │         │ (Google Sign-In) │         │ /auth/google     │
└────┬─────┘         └────────┬─────────┘         └────────┬─────────┘
     │                        │                            │
     │  1. signInWithPopup()  │                            │
     ├───────────────────────►│                            │
     │                        │                            │
     │  2. Google ID token    │                            │
     │◄───────────────────────┤                            │
     │                        │                            │
     │  3. POST /auth/google { idToken }                   │
     ├────────────────────────────────────────────────────►│
     │                                                     │
     │                        4. Verify ID token           │
     │                           Upsert users/{uid}        │
     │                           Mint app JWT              │
     │                                                     │
     │  5. { jwt, uid, isNewUser }                         │
     │◄────────────────────────────────────────────────────┤
     │                                                     │
     │  If isNewUser (WASM runs locally):                  │
     │    6. generate_identity()            → IndexedDB    │
     │    7. generate_signed_prekey()       → IndexedDB    │
     │    8. generate_one_time_prekeys(0,20)→ IndexedDB    │
     │                                                     │
     │  9. POST /keys/identity { identityPublicKey, signingPublicKey }
     ├────────────────────────────────────────────────────►│
     │                                                     │
     │ 10. POST /keys/prekeys { signedPreKey, oneTimePreKeys: [...] }
     ├────────────────────────────────────────────────────►│
     │                                                     │
```

---

## Terraform / OpenTofu Module Structure

```
infra/
├── main.tf                          # Root module — wires everything together
├── variables.tf                     # Project ID, region, environment
├── outputs.tf                       # Function URLs, project number
├── terraform.tfvars                 # Actual values (gitignored)
│
├── modules/
│   ├── project-services/            # Enable required GCP APIs
│   │   ├── main.tf
│   │   └── variables.tf
│   │
│   ├── identity-platform/           # Identity Platform config + Google provider
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   │
│   ├── firestore/                   # Database + indexes
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   │
│   ├── secret-manager/              # VAPID keys, JWT secret
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   │
│   ├── cloud-functions/             # Function deployments + IAM + source bucket
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   │
│   └── iam/                         # Service accounts + role bindings
│       ├── main.tf
│       ├── variables.tf
│       └── outputs.tf
│
└── environments/
    ├── dev.tfvars
    └── prod.tfvars
```

### Required GCP APIs

```hcl
# modules/project-services/main.tf
locals {
  apis = [
    "cloudfunctions.googleapis.com",
    "run.googleapis.com",               # CF 2nd gen runs on Cloud Run
    "firestore.googleapis.com",
    "secretmanager.googleapis.com",
    "identitytoolkit.googleapis.com",   # Identity Platform
    "cloudbuild.googleapis.com",        # CF deployment builds
    "artifactregistry.googleapis.com",  # CF container images
    "logging.googleapis.com",
    "iam.googleapis.com",
  ]
}
```

### IAM Summary

| Principal | Role | Scope |
|---|---|---|
| CF Service Account | `roles/datastore.user` | Project (Firestore read/write) |
| CF Service Account | `roles/secretmanager.secretAccessor` | Secret resources |
| CF Service Account | `roles/logging.logWriter` | Project |
| `allUsers` | `roles/run.invoker` | CF services (public HTTPS) |

---

## Project Structure (Full)

```
gcp_message/
├── ARCHITECTURE.md                  # This file
│
├── infra/                           # Terraform / OpenTofu (GCP resources)
│   └── ...                          # (see module structure above)
│
├── pwa/                             # PWA frontend + WASM crypto
│   ├── crates/
│   │   └── messenger-crypto/        # Rust crate → WASM
│   │       ├── Cargo.toml
│   │       └── src/
│   │           ├── lib.rs
│   │           ├── identity.rs
│   │           ├── x3dh.rs
│   │           ├── double_ratchet.rs
│   │           ├── session.rs
│   │           ├── keys.rs
│   │           ├── store.rs
│   │           └── error.rs
│   │
│   ├── pkg/                         # wasm-pack output (generated, gitignored)
│   │
│   ├── src/                         # TypeScript (Vite / SvelteKit / plain)
│   │   ├── crypto-bridge.ts         # Loads WASM, manages IndexedDB
│   │   ├── auth.ts                  # Identity Platform sign-in
│   │   ├── api.ts                   # HTTP client for Cloud Functions
│   │   ├── push.ts                  # Web Push subscription management
│   │   └── ...
│   │
│   ├── public/
│   │   ├── manifest.json            # PWA manifest
│   │   └── sw.js                    # Service Worker (push + cache)
│   │
│   ├── package.json
│   └── build.sh                     # wasm-pack build + vite build
│
└── functions/                       # Cloud Functions source
    ├── package.json                 # (or requirements.txt for Python)
    └── src/
        ├── auth.ts
        ├── keys.ts
        ├── chat.ts
        └── push.ts
```

---

## Cost Estimate (PoC / Low Traffic)

| Service | Free Tier | Est. Monthly Cost |
|---|---|---|
| Cloud Functions | 2M invocations, 400K GB-sec | $0 |
| Firestore | 1 GiB, 50K reads/day, 20K writes/day | $0 |
| Secret Manager | 6 versions, 10K accesses | $0 |
| Identity Platform | 50K MAU (Google provider) | $0 |
| Cloud Logging | 50 GiB/month ingestion | $0 |
| **Total** | | **$0** (within free tier) |

---

## Security Considerations

| Concern | Mitigation |
|---|---|
| Server sees plaintext | Impossible — encryption/decryption happens only in WASM on client |
| Stolen database | Attacker gets only ciphertext + public keys — useless without private keys |
| Compromised long-term key | Double Ratchet limits damage: only current ratchet step exposed, self-heals after round-trip |
| Prekey exhaustion | If no OPKs left, X3DH falls back to 3-DH (still secure, but weaker against replay). Client auto-replenishes. |
| VAPID key leak | Attacker could send push notifications but NOT read messages (push payload has no plaintext) |
| JWT secret leak | Attacker could forge auth tokens — mitigate with short expiry (1h) + rotation via Secret Manager versions |
| Lost private key | User loses access to message history — PoC accepts this; future: key backup with passphrase wrapping |
| Push payload privacy | Push notifications contain only "new message" hint + roomId, never message content |
| WASM integrity | WASM module served over HTTPS + SRI hash in script tag. Subresource integrity prevents tampering. |
| Out-of-order messages | Double Ratchet maintains skipped message keys (up to configurable window, e.g. 1000) |
| Identity verification (TOFU) | Client stores first-seen identity key per user. Warn if key changes (like SSH known_hosts). |

---

## Future Enhancements (Post-PoC)

- **Group chats** — Sender Keys (Signal's approach) or MLS protocol for efficient group encryption
- **Key backup** — passphrase-wrapped key export/import (Argon2id KDF + AES-256-GCM)
- **Multi-device sync** — device-to-device key transfer via QR code / secure pairing
- **File/media messages** — encrypt files in WASM, upload to Cloud Storage, store download URL in message
- **Message expiry** — TTL on Firestore documents (Firestore TTL policy)
- **Custom domain** — Cloud Run domain mapping or Cloud Load Balancer in front of functions
- **Rate limiting** — Cloud Armor or in-function rate limiter
- **Monitoring** — Cloud Monitoring dashboards + alerting policies
- **Safety number** — display a fingerprint of both users' identity keys for out-of-band verification
