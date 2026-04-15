# End-to-End Validation Guide

## Prerequisites

- A deployed Cloud Functions environment (dev or staging).
- Two test Google accounts (Alice and Bob).
- WASM build in `apps/pwa/pkg/` (`cd crates/messenger-crypto && ./build-wasm.sh`).
- PWA running locally or deployed (`npm run dev` in `apps/pwa/`).

---

## Step 1 — Authentication

1. Open the PWA in a browser.
2. Obtain a Google ID token using the browser console:
   ```javascript
   // Using Firebase Auth
   const credential = await signInWithPopup(auth, new GoogleAuthProvider());
   const idToken = await credential.user.getIdToken();
   console.log(idToken);
   ```
3. Enter the ID token in the sign-in form.
4. Verify response: `{ jwt: "...", uid: "...", isNewUser: true/false }`.
5. On first login, verify that:
   - Identity keys are stored in IndexedDB (`Application → IndexedDB → messenger`).
   - A new `preKeyBundles` document appears in Firestore for your UID.

---

## Step 2 — Key publication check

```bash
JWT="<jwt from step 1>"
UID="<your uid>"
BASE_URL="https://<region>-<project>.cloudfunctions.net"

curl -s \
  -H "Authorization: Bearer ${JWT}" \
  "${BASE_URL}/keys/bundle?uid=${UID}&countOnly=true"
# Expected: { "oneTimePreKeyCount": 20 }
```

---

## Step 3 — First encrypted message (X3DH)

Using Alice and Bob accounts:

1. Alice signs in → note `jwt_alice` and `uid_alice`.
2. Bob signs in → note `jwt_bob` and `uid_bob`.
3. Alice fetches Bob's prekey bundle:
   ```bash
   curl -s -H "Authorization: Bearer ${jwt_alice}" \
     "${BASE_URL}/keys/bundle?uid=${uid_bob}"
   # Expected: { identityPublicKey, signingPublicKey, signedPreKey, oneTimePreKey }
   ```
4. In the PWA, Alice sends a message to Bob. The PWA will:
   - Run X3DH initiate via WASM.
   - POST `/chat/send` with `type: "x3dh_initial"`.
5. Bob opens the PWA and views the conversation. The PWA will:
   - Run X3DH respond via WASM.
   - Decrypt the message with `ratchetDecrypt`.
6. Verify Bob sees the plaintext.

---

## Step 4 — Subsequent ratcheted messages

1. Alice sends a second message to Bob.
2. Verify `type: "ratchet"` in the network request (no ephemeral key fields).
3. Bob decrypts successfully.
4. Send 5+ messages in each direction; verify all decrypt correctly.

---

## Step 5 — Push notification receipt

1. Bob enables notifications in the browser.
2. Alice sends a message to Bob.
3. Verify Bob receives a push notification with:
   - Title and body fields present.
   - **No plaintext message content** in the payload.
4. Click the notification and verify it opens the correct room.

---

## Step 6 — Post-deployment verification artifacts

After completing the above steps, capture the following as evidence:

- [ ] Screenshot of Firestore showing `rooms/{roomId}/messages` with opaque `header` + `ciphertext`.
- [ ] Screenshot of Firestore showing `preKeyBundles/{uid}/oneTimePreKeys` decremented after use.
- [ ] Network trace showing `/chat/send` request with no plaintext body.
- [ ] Browser DevTools showing IndexedDB `sessions` entry populated.
- [ ] Push notification screenshot showing hint-only payload.
