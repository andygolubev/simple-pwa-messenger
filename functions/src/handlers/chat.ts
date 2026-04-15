import { Request, Response } from "express";
import { FieldValue } from "@google-cloud/firestore";
import crypto from "crypto";
import { getDb } from "../utils/firestore";
import { AuthenticatedRequest } from "../middleware/auth";
import { sendPushNotifications } from "./push";

function deriveRoomId(uid1: string, uid2: string): string {
  const sorted = [uid1, uid2].sort().join(":");
  return crypto.createHash("sha256").update(sorted).digest("hex");
}

interface SendMessageRequest {
  roomId: string;
  recipientUid: string;
  type: "x3dh_initial" | "ratchet";
  header: string;
  ciphertext: string;
  ephemeralPublic?: string;
  usedOneTimePreKeyId?: number;
}

export async function handleSendMessage(
  req: AuthenticatedRequest,
  res: Response
): Promise<void> {
  const senderUid = req.uid!;
  const body = req.body as SendMessageRequest;

  const { roomId, recipientUid, type, header, ciphertext } = body;
  if (!roomId || !recipientUid || !type || !header || !ciphertext) {
    res.status(400).json({ error: "roomId, recipientUid, type, header, and ciphertext are required" });
    return;
  }
  if (type !== "x3dh_initial" && type !== "ratchet") {
    res.status(400).json({ error: "type must be x3dh_initial or ratchet" });
    return;
  }
  if (type === "x3dh_initial" && !body.ephemeralPublic) {
    res.status(400).json({ error: "ephemeralPublic is required for x3dh_initial" });
    return;
  }

  const db = getDb();

  // Verify recipient exists
  const recipientSnap = await db.collection("users").doc(recipientUid).get();
  if (!recipientSnap.exists) {
    res.status(404).json({ error: "Recipient does not exist" });
    return;
  }

  const serverTimestamp = new Date().toISOString();
  const messageData: Record<string, unknown> = {
    senderUid,
    type,
    header,
    ciphertext,
    createdAt: serverTimestamp,
    serverTimestamp,
  };

  if (type === "x3dh_initial") {
    messageData.ephemeralPublic = body.ephemeralPublic;
    messageData.usedOneTimePreKeyId = body.usedOneTimePreKeyId ?? null;
  }

  // Atomically consume OTP key if this is an X3DH initial message
  if (type === "x3dh_initial" && body.usedOneTimePreKeyId !== undefined) {
    const bundleRef = db.collection("preKeyBundles").doc(recipientUid);
    await db.runTransaction(async (tx) => {
      const snap = await tx.get(bundleRef);
      if (snap.exists) {
        const keys: Array<{ id: number; publicKey: string }> =
          snap.data()?.oneTimePreKeys ?? [];
        const consumed = keys.find((k) => k.id === body.usedOneTimePreKeyId);
        if (consumed) {
          tx.update(bundleRef, {
            oneTimePreKeys: FieldValue.arrayRemove(consumed),
          });
        }
      }
    });
  }

  // Ensure room document exists
  const roomRef = db.collection("rooms").doc(roomId);
  await roomRef.set(
    {
      participants: [senderUid, recipientUid].sort(),
      lastMessageAt: serverTimestamp,
    },
    { merge: true }
  );

  const messageRef = await roomRef.collection("messages").add(messageData);

  // Fan-out push notification (best-effort, do not fail the request)
  sendPushNotifications(recipientUid, {
    title: "New message",
    body: "You have a new encrypted message",
    roomId,
  }).catch(() => {
    // Stale subscriptions cleaned up inside sendPushNotifications
  });

  res.json({ messageId: messageRef.id, serverTimestamp });
}

export async function handleGetHistory(
  req: AuthenticatedRequest,
  res: Response
): Promise<void> {
  const uid = req.uid!;
  const { roomId, limit: limitStr, after } = req.query as {
    roomId?: string;
    limit?: string;
    after?: string;
  };

  if (!roomId) {
    res.status(400).json({ error: "roomId is required" });
    return;
  }

  const db = getDb();
  const roomSnap = await db.collection("rooms").doc(roomId).get();
  if (!roomSnap.exists) {
    res.status(404).json({ error: "Room not found" });
    return;
  }

  const roomData = roomSnap.data()!;
  const participants: string[] = roomData.participants ?? [];
  if (!participants.includes(uid)) {
    res.status(403).json({ error: "Not a participant of this room" });
    return;
  }

  const limit = Math.min(parseInt(limitStr ?? "50", 10), 100);
  let query = db
    .collection("rooms")
    .doc(roomId)
    .collection("messages")
    .orderBy("createdAt", "asc")
    .limit(limit + 1);

  if (after) {
    query = query.startAfter(after);
  }

  const snap = await query.get();
  const docs = snap.docs;
  const hasMore = docs.length > limit;
  const messages = docs.slice(0, limit).map((doc) => ({
    messageId: doc.id,
    ...doc.data(),
  }));

  res.json({
    messages,
    nextCursor: hasMore ? messages[messages.length - 1].messageId : null,
  });
}

export async function handlePollRooms(
  req: AuthenticatedRequest,
  res: Response
): Promise<void> {
  const uid = req.uid!;
  const { since } = req.query as { since?: string };

  if (!since) {
    res.status(400).json({ error: "since is required" });
    return;
  }

  const db = getDb();
  const snap = await db
    .collection("rooms")
    .where("participants", "array-contains", uid)
    .where("lastMessageAt", ">", since)
    .orderBy("lastMessageAt", "desc")
    .get();

  const rooms = snap.docs.map((doc) => {
    const data = doc.data();
    return {
      roomId: doc.id,
      participants: data.participants ?? [],
      lastMessageAt: data.lastMessageAt,
    };
  });

  res.json({ rooms });
}

// Helper: compute deterministic room ID for 1:1 chats
export { deriveRoomId };
