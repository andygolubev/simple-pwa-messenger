/**
 * Encrypted message send/receive pipeline.
 *
 * First message in a new session:
 *   - Fetch recipient's prekey bundle
 *   - Run X3DH initiate (via WASM) → session state + ephemeral key
 *   - Encrypt plaintext with ratchet_encrypt
 *   - POST /chat/send with type=x3dh_initial
 *
 * Subsequent messages:
 *   - Load existing session state from IndexedDB
 *   - Encrypt with ratchet_encrypt
 *   - POST /chat/send with type=ratchet
 *
 * Receiving messages:
 *   - For x3dh_initial: run X3DH respond → session state, then ratchet_decrypt
 *   - For ratchet: load session, ratchet_decrypt
 */

import * as cryptoBridge from "../crypto/bridge";
import * as db from "../store/db";
import * as api from "../api/client";
import { deriveRoomId } from "./room";

export interface DecryptedMessage {
  messageId: string;
  senderUid: string;
  plaintext: string;
  createdAt: string;
}

/**
 * Send a plaintext message to a recipient.
 * Establishes a new X3DH session if one does not already exist.
 */
export async function sendPlaintextMessage(
  myUid: string,
  recipientUid: string,
  plaintext: string,
  jwt: string
): Promise<{ messageId: string; serverTimestamp: string }> {
  const roomId = await deriveRoomId(myUid, recipientUid);
  const identity = await db.loadIdentity(myUid);
  if (!identity) throw new Error("Identity not found — bootstrapKeys must be called first");

  let sessionState: string;
  let messageType: "x3dh_initial" | "ratchet";
  let ephemeralPublic: string | undefined;
  let usedOneTimePreKeyId: number | undefined;

  const existingSession = await db.loadSession(roomId);

  if (!existingSession) {
    // First message: perform X3DH handshake
    const bundle = await api.getPreKeyBundle(jwt, recipientUid);

    const opkJson = bundle.oneTimePreKey
      ? JSON.stringify({
          id: bundle.oneTimePreKey.id,
          publicKey: bundle.oneTimePreKey.publicKey,
          privateKey: "", // server only returns public half
        })
      : undefined;

    const x3dhResult = await cryptoBridge.x3dhInitiate(
      identity.identityPrivateKey,
      bundle.identityPublicKey,
      bundle.signingPublicKey,
      bundle.signedPreKey.publicKey,
      bundle.signedPreKey.signature,
      opkJson
    );

    sessionState = x3dhResult.sessionState;
    ephemeralPublic = x3dhResult.ephemeralPublic;
    usedOneTimePreKeyId = x3dhResult.usedOneTimePreKeyId;
    messageType = "x3dh_initial";
  } else {
    sessionState = existingSession.sessionState;
    messageType = "ratchet";
  }

  // Encrypt the message
  const encrypted = await cryptoBridge.ratchetEncrypt(sessionState, plaintext);

  // Persist the updated session state
  await db.saveSession({
    roomId,
    peerUid: recipientUid,
    sessionState: encrypted.updatedSessionState,
    updatedAt: Date.now(),
  });

  // Send to server
  return api.sendMessage(jwt, {
    roomId,
    recipientUid,
    type: messageType,
    header: encrypted.message.header,
    ciphertext: encrypted.message.ciphertext,
    ephemeralPublic,
    usedOneTimePreKeyId,
  });
}

/**
 * Decrypt a received encrypted message.
 * Establishes a responder session on the first X3DH message.
 */
export async function decryptMessage(
  myUid: string,
  message: api.EncryptedMessage,
  jwt: string
): Promise<DecryptedMessage> {
  const roomId = await deriveRoomId(myUid, message.senderUid);
  const identity = await db.loadIdentity(myUid);
  if (!identity) throw new Error("Identity not found");

  let sessionState: string;
  const existingSession = await db.loadSession(roomId);

  if (!existingSession && message.type === "x3dh_initial") {
    if (!message.ephemeralPublic) {
      throw new Error("x3dh_initial message is missing ephemeralPublic");
    }

    // Find the matching signed prekey
    const signedPreKey = await db.loadLatestSignedPreKey();
    if (!signedPreKey) throw new Error("Signed prekey not found");

    // Find the matching one-time prekey (if any)
    let opkPrivate: string | undefined;
    if (message.usedOneTimePreKeyId !== undefined) {
      const opk = await db.getOneTimePreKeyPrivate(message.usedOneTimePreKeyId);
      if (opk) {
        opkPrivate = opk.privateKey;
        await db.deleteOneTimePreKey(message.usedOneTimePreKeyId);
      }
    }

    const x3dhResult = await cryptoBridge.x3dhRespond(
      identity.identityPrivateKey,
      signedPreKey.privateKey,
      opkPrivate,
      message.senderUid, // NOTE: server stores sender's identity public key in the message
      message.ephemeralPublic
    );

    sessionState = x3dhResult.sessionState;
  } else if (existingSession) {
    sessionState = existingSession.sessionState;
  } else {
    throw new Error(
      `No session for room ${roomId} and message type is ${message.type} — cannot decrypt`
    );
  }

  // Decrypt the message
  const decrypted = await cryptoBridge.ratchetDecrypt(
    sessionState,
    message.header,
    message.ciphertext
  );

  // Persist updated session state
  await db.saveSession({
    roomId,
    peerUid: message.senderUid,
    sessionState: decrypted.updatedSessionState,
    updatedAt: Date.now(),
  });

  return {
    messageId: message.messageId,
    senderUid: message.senderUid,
    plaintext: decrypted.plaintext,
    createdAt: message.createdAt,
  };
}
