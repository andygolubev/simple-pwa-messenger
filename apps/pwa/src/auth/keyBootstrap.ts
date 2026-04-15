/**
 * Key bootstrap flow — called on first login when the server signals isNewUser=true.
 *
 * 1. Generate identity key pair (X25519) + signing key pair (Ed25519) via WASM.
 * 2. Publish identity public keys to the server.
 * 3. Generate a signed prekey and 20 one-time prekeys.
 * 4. Upload prekeys to the server.
 * 5. Persist all private key material to IndexedDB.
 */

import * as crypto from "../crypto/bridge";
import * as db from "../store/db";
import * as api from "../api/client";

const INITIAL_OPK_COUNT = 20;
const OPK_REPLENISH_THRESHOLD = 5;
const OPK_REPLENISH_BATCH = 20;
const SPK_ROTATION_INTERVAL_MS = 30 * 24 * 60 * 60 * 1000; // 30 days

export async function bootstrapKeys(uid: string, jwt: string): Promise<void> {
  // 1. Generate identity
  const identity = await crypto.generateIdentity();

  // 2. Generate signed prekey (id=1 on first login)
  const signedPreKey = await crypto.generateSignedPrekey(identity.signingPrivateKey, 1);

  // 3. Generate one-time prekeys
  const oneTimePreKeys = await crypto.generateOneTimePrekeys(1, INITIAL_OPK_COUNT);

  // 4. Persist private material to IndexedDB
  await db.saveIdentity({
    uid,
    identityPrivateKey: identity.identityPrivateKey,
    identityPublicKey: identity.identityPublicKey,
    signingPrivateKey: identity.signingPrivateKey,
    signingPublicKey: identity.signingPublicKey,
    createdAt: Date.now(),
  });

  await db.saveSignedPreKey({
    id: signedPreKey.id,
    publicKey: signedPreKey.publicKey,
    privateKey: signedPreKey.privateKey,
    signature: signedPreKey.signature,
    createdAt: Date.now(),
  });

  await db.saveOneTimePreKeys(
    oneTimePreKeys.map((k) => ({
      id: k.id,
      publicKey: k.publicKey,
      privateKey: k.privateKey,
    }))
  );

  // 5. Publish public keys to server
  await api.publishIdentityKeys(jwt, {
    identityPublicKey: identity.identityPublicKey,
    signingPublicKey: identity.signingPublicKey,
  });

  await api.uploadPrekeys(
    jwt,
    { id: signedPreKey.id, publicKey: signedPreKey.publicKey, signature: signedPreKey.signature },
    oneTimePreKeys.map((k) => ({ id: k.id, publicKey: k.publicKey }))
  );
}

/**
 * Check if prekeys need replenishment and rotate/upload as needed.
 * Should be called periodically (e.g., on app resume or after each successful auth).
 */
export async function checkAndReplenishPrekeys(
  uid: string,
  jwt: string
): Promise<void> {
  const identity = await db.loadIdentity(uid);
  if (!identity) return;

  // Check OPK count
  const serverCount = await api.getOwnPreKeyCount(jwt, uid);
  if (serverCount < OPK_REPLENISH_THRESHOLD) {
    const nextId = await db.getNextOneTimePreKeyId();
    const newKeys = await crypto.generateOneTimePrekeys(nextId, OPK_REPLENISH_BATCH);

    await db.saveOneTimePreKeys(
      newKeys.map((k) => ({ id: k.id, publicKey: k.publicKey, privateKey: k.privateKey }))
    );

    await api.uploadPrekeys(
      jwt,
      undefined,
      newKeys.map((k) => ({ id: k.id, publicKey: k.publicKey }))
    );
  }

  // Check if signed prekey needs rotation (every 30 days)
  const currentSpk = await db.loadLatestSignedPreKey();
  if (!currentSpk || Date.now() - currentSpk.createdAt > SPK_ROTATION_INTERVAL_MS) {
    const nextSpkId = currentSpk ? currentSpk.id + 1 : 1;
    const newSpk = await crypto.generateSignedPrekey(identity.signingPrivateKey, nextSpkId);

    await db.saveSignedPreKey({
      id: newSpk.id,
      publicKey: newSpk.publicKey,
      privateKey: newSpk.privateKey,
      signature: newSpk.signature,
      createdAt: Date.now(),
    });

    await api.uploadPrekeys(jwt, {
      id: newSpk.id,
      publicKey: newSpk.publicKey,
      signature: newSpk.signature,
    });
  }
}
