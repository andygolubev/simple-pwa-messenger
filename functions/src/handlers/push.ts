import { Request, Response } from "express";
import webpush from "web-push";
import crypto from "crypto";
import { getDb } from "../utils/firestore";
import { getVapidKeys } from "../utils/secrets";
import { AuthenticatedRequest } from "../middleware/auth";

interface PushSubscriptionKeys {
  p256dh: string;
  auth: string;
}

interface StoredSubscription {
  deviceId: string;
  endpoint: string;
  keys: PushSubscriptionKeys;
  userAgent?: string;
  createdAt: string;
}

async function initVapid(): Promise<void> {
  const { publicKey, privateKey } = await getVapidKeys();
  webpush.setVapidDetails("mailto:admin@yourdomain.com", publicKey, privateKey);
}

export async function handleSubscribePush(
  req: AuthenticatedRequest,
  res: Response
): Promise<void> {
  const uid = req.uid!;
  const { endpoint, keys, userAgent } = req.body as {
    endpoint?: string;
    keys?: PushSubscriptionKeys;
    userAgent?: string;
  };

  if (!endpoint || !keys?.p256dh || !keys?.auth) {
    res.status(400).json({ error: "endpoint and keys (p256dh, auth) are required" });
    return;
  }

  const deviceId = crypto
    .createHash("sha256")
    .update(endpoint)
    .digest("hex")
    .slice(0, 16);

  const db = getDb();
  const subRef = db
    .collection("pushSubscriptions")
    .doc(uid)
    .collection("devices")
    .doc(deviceId);

  const subscription: StoredSubscription = {
    deviceId,
    endpoint,
    keys,
    userAgent,
    createdAt: new Date().toISOString(),
  };

  await subRef.set(subscription);
  res.json({ deviceId });
}

export async function handleUnsubscribePush(
  req: AuthenticatedRequest,
  res: Response
): Promise<void> {
  const uid = req.uid!;
  const { endpoint } = req.body as { endpoint?: string };

  if (!endpoint) {
    res.status(400).json({ error: "endpoint is required" });
    return;
  }

  const deviceId = crypto
    .createHash("sha256")
    .update(endpoint)
    .digest("hex")
    .slice(0, 16);

  const db = getDb();
  const subRef = db
    .collection("pushSubscriptions")
    .doc(uid)
    .collection("devices")
    .doc(deviceId);

  const snap = await subRef.get();
  if (!snap.exists) {
    res.status(404).json({ error: "Subscription not found" });
    return;
  }

  await subRef.delete();
  res.json({ ok: true });
}

export async function sendPushNotifications(
  recipientUid: string,
  hint: { title: string; body: string; roomId: string }
): Promise<void> {
  await initVapid();
  const db = getDb();

  const devicesSnap = await db
    .collection("pushSubscriptions")
    .doc(recipientUid)
    .collection("devices")
    .get();

  const payload = JSON.stringify(hint);
  const staleRefs: FirebaseFirestore.DocumentReference[] = [];

  await Promise.all(
    devicesSnap.docs.map(async (doc) => {
      const sub = doc.data() as StoredSubscription;
      try {
        await webpush.sendNotification(
          { endpoint: sub.endpoint, keys: sub.keys },
          payload
        );
      } catch (err: unknown) {
        const status = (err as { statusCode?: number }).statusCode;
        if (status === 410 || status === 404) {
          staleRefs.push(doc.ref);
        }
      }
    })
  );

  // Clean up stale subscriptions
  await Promise.all(staleRefs.map((ref) => ref.delete()));
}
