import { Request, Response } from "express";
import admin from "firebase-admin";
import { getDb } from "../utils/firestore";
import { mintJwt } from "../middleware/auth";

let adminInitialized = false;

function ensureAdmin(): void {
  if (!adminInitialized) {
    admin.initializeApp();
    adminInitialized = true;
  }
}

export async function handleAuthGoogle(
  req: Request,
  res: Response
): Promise<void> {
  const { idToken } = req.body as { idToken?: string };
  if (!idToken || typeof idToken !== "string") {
    res.status(400).json({ error: "idToken is required" });
    return;
  }

  ensureAdmin();

  let decodedToken: admin.auth.DecodedIdToken;
  try {
    decodedToken = await admin.auth().verifyIdToken(idToken);
  } catch {
    res.status(401).json({ error: "Invalid or expired Google ID token" });
    return;
  }

  const uid = decodedToken.uid;
  const db = getDb();
  const userRef = db.collection("users").doc(uid);

  const snap = await userRef.get();
  const isNewUser = !snap.exists;

  await userRef.set(
    {
      uid,
      email: decodedToken.email ?? null,
      displayName: decodedToken.name ?? null,
      updatedAt: new Date().toISOString(),
      ...(isNewUser ? { createdAt: new Date().toISOString() } : {}),
    },
    { merge: true }
  );

  const appJwt = await mintJwt(uid);
  res.json({ jwt: appJwt, uid, isNewUser });
}
