import { Request, Response } from "express";
import { FieldValue } from "@google-cloud/firestore";
import { getDb } from "../utils/firestore";
import { AuthenticatedRequest } from "../middleware/auth";

export async function handlePublishIdentityKeys(
  req: AuthenticatedRequest,
  res: Response
): Promise<void> {
  const uid = req.uid!;
  const { identityPublicKey, signingPublicKey } = req.body as {
    identityPublicKey?: string;
    signingPublicKey?: string;
  };

  if (!identityPublicKey || !signingPublicKey) {
    res.status(400).json({ error: "identityPublicKey and signingPublicKey are required" });
    return;
  }

  const db = getDb();
  await db.collection("preKeyBundles").doc(uid).set(
    {
      uid,
      identityPublicKey,
      signingPublicKey,
      updatedAt: new Date().toISOString(),
    },
    { merge: true }
  );

  res.json({ ok: true });
}

interface SignedPreKeyUpload {
  id: number;
  publicKey: string;
  signature: string;
}

interface OneTimePreKeyUpload {
  id: number;
  publicKey: string;
}

export async function handleUploadPrekeys(
  req: AuthenticatedRequest,
  res: Response
): Promise<void> {
  const uid = req.uid!;
  const { signedPreKey, oneTimePreKeys } = req.body as {
    signedPreKey?: SignedPreKeyUpload;
    oneTimePreKeys?: OneTimePreKeyUpload[];
  };

  if (!signedPreKey && (!oneTimePreKeys || oneTimePreKeys.length === 0)) {
    res.status(400).json({ error: "signedPreKey or oneTimePreKeys is required" });
    return;
  }

  const db = getDb();
  const bundleRef = db.collection("preKeyBundles").doc(uid);

  const updates: Record<string, unknown> = {
    updatedAt: new Date().toISOString(),
  };

  if (signedPreKey) {
    updates.signedPreKey = signedPreKey;
  }

  await db.runTransaction(async (tx) => {
    const snap = await tx.get(bundleRef);
    const existing = snap.data() ?? {};

    if (oneTimePreKeys && oneTimePreKeys.length > 0) {
      const currentKeys: OneTimePreKeyUpload[] = existing.oneTimePreKeys ?? [];
      const existingIds = new Set(currentKeys.map((k) => k.id));
      const newKeys = oneTimePreKeys.filter((k) => !existingIds.has(k.id));
      updates.oneTimePreKeys = FieldValue.arrayUnion(...newKeys) as unknown;
    }

    tx.set(bundleRef, updates, { merge: true });
  });

  const snap = await bundleRef.get();
  const data = snap.data();
  const oneTimePreKeyCount: number = (data?.oneTimePreKeys ?? []).length;

  res.json({ ok: true, oneTimePreKeyCount });
}

export async function handleGetPreKeyBundle(
  req: AuthenticatedRequest,
  res: Response
): Promise<void> {
  const { uid: targetUid, countOnly } = req.query as {
    uid?: string;
    countOnly?: string;
  };

  if (!targetUid) {
    res.status(400).json({ error: "uid query parameter is required" });
    return;
  }

  const db = getDb();
  const bundleRef = db.collection("preKeyBundles").doc(targetUid);

  if (countOnly === "true") {
    const snap = await bundleRef.get();
    if (!snap.exists) {
      res.status(404).json({ error: "User has no prekey bundle" });
      return;
    }
    const data = snap.data()!;
    const oneTimePreKeyCount: number = (data.oneTimePreKeys ?? []).length;
    res.json({ oneTimePreKeyCount });
    return;
  }

  let poppedKey: OneTimePreKeyUpload | null = null;

  await db.runTransaction(async (tx) => {
    const snap = await tx.get(bundleRef);
    if (!snap.exists) {
      return;
    }
    const data = snap.data()!;
    const keys: OneTimePreKeyUpload[] = data.oneTimePreKeys ?? [];
    if (keys.length > 0) {
      poppedKey = keys[keys.length - 1];
      tx.update(bundleRef, {
        oneTimePreKeys: FieldValue.arrayRemove(poppedKey),
      });
    }
  });

  const snap = await bundleRef.get();
  if (!snap.exists) {
    res.status(404).json({ error: "User not found or no prekey bundle published" });
    return;
  }

  const data = snap.data()!;
  if (!data.identityPublicKey || !data.signedPreKey) {
    res.status(404).json({ error: "Prekey bundle not fully initialized" });
    return;
  }

  res.json({
    identityPublicKey: data.identityPublicKey,
    signingPublicKey: data.signingPublicKey,
    signedPreKey: data.signedPreKey,
    oneTimePreKey: poppedKey,
  });
}
