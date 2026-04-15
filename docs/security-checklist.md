# Security Checklist

## JWT rotation

- [ ] JWT signing key is stored in Secret Manager (never in code or environment files).
- [ ] JWT expiry is set to 1 hour (`expiresIn: "1h"` in `functions/src/middleware/auth.ts`).
- [ ] To rotate: add a new Secret Manager version, update function runtime to pick up
      the new version on next cold start (or force redeploy).
- [ ] Old secret versions are scheduled for destruction after rotation grace period
      (`secret_version_destroy_ttl` in Terraform).

## Secret access scope

- [ ] Function service account is granted `secretmanager.secretAccessor` only on the
      specific secrets it needs (`jwt-signing-key`, `vapid-private-key`, `vapid-public-key`).
- [ ] No wildcard `*` Secret Manager bindings exist.
- [ ] Service account has no `editor` or `owner` roles — only the minimum permissions
      defined in `infra/modules/iam/`.
- [ ] IAM bindings are managed via Terraform, not via `gcloud` ad-hoc commands.

## TOFU (Trust On First Use) identity warnings

- [ ] X3DH signed prekey signature is always verified by the client before initiating
      a session (`verify_signed_prekey_signature` in `crates/messenger-crypto/src/identity.rs`).
- [ ] The client displays a "safety number" or out-of-band verification prompt to users
      on first contact with a new peer (not yet implemented in PWA — tracked as post-PoC).
- [ ] Server-side identity binding between UID and prekey bundle is enforced by JWT
      authentication — only the owner can update their own prekey bundle.

## Push payload plaintext prohibition

- [ ] `sendPushNotifications` in `functions/src/handlers/push.ts` sends only
      `{ title, body, roomId }` — never message ciphertext or sender identity.
- [ ] Service worker (`apps/pwa/public/sw.js`) reads only hint fields from the push
      payload — it cannot access message content.
- [ ] Review all call sites of `sendPushNotifications` when adding new message types.

## Firestore security

- [ ] Only the function service account has read/write access to Firestore.
- [ ] Client browsers have no direct Firestore access (all access goes through the
      Cloud Functions API with JWT validation).
- [ ] Firestore security rules (if using Firebase SDK) should deny all direct client access.

## Key material handling in WASM

- [ ] Private keys are zeroized on drop (`#[derive(Zeroize, ZeroizeOnDrop)]`) in
      `crates/messenger-crypto/src/keys.rs`.
- [ ] Private keys are never sent to the server — only public keys are uploaded via the API.
- [ ] Session state stored in IndexedDB contains symmetric chain keys — ensure the
      storage is origin-scoped and not accessible by third-party scripts.

## Content Security Policy (PWA)

- [ ] Set a CSP header that disallows `eval()` and restricts script sources to the PWA origin.
- [ ] Do not inline script tags in `index.html`.

## Dependency auditing

- [ ] Run `cargo audit` on the Rust crate before each deployment.
- [ ] Run `npm audit` on both `functions/` and `apps/pwa/` before each deployment.
- [ ] Pin major dependency versions; review minor/patch bumps for breaking changes in
      cryptographic dependencies (`aes-gcm`, `ed25519-dalek`, `x25519-dalek`).
