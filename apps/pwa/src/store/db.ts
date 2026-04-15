/**
 * IndexedDB persistence layer for keys, sessions, and auth state.
 * Uses the `idb` library for typed access.
 */
import { openDB, type DBSchema, type IDBPDatabase } from "idb";

export interface StoredIdentity {
  uid: string;
  identityPrivateKey: string;
  identityPublicKey: string;
  signingPrivateKey: string;
  signingPublicKey: string;
  createdAt: number;
}

export interface StoredSignedPreKey {
  id: number;
  publicKey: string;
  privateKey: string;
  signature: string;
  createdAt: number;
}

export interface StoredOneTimePreKey {
  id: number;
  publicKey: string;
  privateKey: string;
}

export interface StoredSession {
  roomId: string;
  peerUid: string;
  sessionState: string;
  updatedAt: number;
}

export interface StoredAuthState {
  id: "current";
  jwt: string;
  uid: string;
  expiresAt: number;
}

interface MessengerDB extends DBSchema {
  identity: {
    key: string;
    value: StoredIdentity;
  };
  signedPreKeys: {
    key: number;
    value: StoredSignedPreKey;
  };
  oneTimePreKeys: {
    key: number;
    value: StoredOneTimePreKey;
  };
  sessions: {
    key: string;
    value: StoredSession;
  };
  auth: {
    key: string;
    value: StoredAuthState;
  };
}

const DB_NAME = "messenger";
const DB_VERSION = 1;

let dbPromise: Promise<IDBPDatabase<MessengerDB>> | null = null;

function getDb(): Promise<IDBPDatabase<MessengerDB>> {
  if (!dbPromise) {
    dbPromise = openDB<MessengerDB>(DB_NAME, DB_VERSION, {
      upgrade(db) {
        db.createObjectStore("identity", { keyPath: "uid" });
        db.createObjectStore("signedPreKeys", { keyPath: "id" });
        db.createObjectStore("oneTimePreKeys", { keyPath: "id" });
        db.createObjectStore("sessions", { keyPath: "roomId" });
        db.createObjectStore("auth", { keyPath: "id" });
      },
    });
  }
  return dbPromise;
}

// ─── Identity ─────────────────────────────────────────────────────────────────

export async function saveIdentity(identity: StoredIdentity): Promise<void> {
  const db = await getDb();
  await db.put("identity", identity);
}

export async function loadIdentity(uid: string): Promise<StoredIdentity | undefined> {
  const db = await getDb();
  return db.get("identity", uid);
}

// ─── Signed prekeys ──────────────────────────────────────────────────────────

export async function saveSignedPreKey(spk: StoredSignedPreKey): Promise<void> {
  const db = await getDb();
  await db.put("signedPreKeys", spk);
}

export async function loadSignedPreKey(id: number): Promise<StoredSignedPreKey | undefined> {
  const db = await getDb();
  return db.get("signedPreKeys", id);
}

export async function loadLatestSignedPreKey(): Promise<StoredSignedPreKey | undefined> {
  const db = await getDb();
  const all = await db.getAll("signedPreKeys");
  if (all.length === 0) return undefined;
  return all.sort((a, b) => b.createdAt - a.createdAt)[0];
}

// ─── One-time prekeys ────────────────────────────────────────────────────────

export async function saveOneTimePreKeys(keys: StoredOneTimePreKey[]): Promise<void> {
  const db = await getDb();
  const tx = db.transaction("oneTimePreKeys", "readwrite");
  for (const key of keys) {
    await tx.store.put(key);
  }
  await tx.done;
}

export async function getOneTimePreKeyPrivate(id: number): Promise<StoredOneTimePreKey | undefined> {
  const db = await getDb();
  return db.get("oneTimePreKeys", id);
}

export async function deleteOneTimePreKey(id: number): Promise<void> {
  const db = await getDb();
  await db.delete("oneTimePreKeys", id);
}

export async function countOneTimePreKeys(): Promise<number> {
  const db = await getDb();
  return db.count("oneTimePreKeys");
}

export async function getNextOneTimePreKeyId(): Promise<number> {
  const db = await getDb();
  const all = await db.getAll("oneTimePreKeys");
  if (all.length === 0) return 1;
  return Math.max(...all.map((k) => k.id)) + 1;
}

// ─── Sessions ─────────────────────────────────────────────────────────────────

export async function saveSession(session: StoredSession): Promise<void> {
  const db = await getDb();
  await db.put("sessions", session);
}

export async function loadSession(roomId: string): Promise<StoredSession | undefined> {
  const db = await getDb();
  return db.get("sessions", roomId);
}

// ─── Auth state ───────────────────────────────────────────────────────────────

export async function saveAuthState(state: Omit<StoredAuthState, "id">): Promise<void> {
  const db = await getDb();
  await db.put("auth", { id: "current", ...state });
}

export async function loadAuthState(): Promise<StoredAuthState | undefined> {
  const db = await getDb();
  const record = await db.get("auth", "current");
  if (!record) return undefined;
  if (Date.now() > record.expiresAt) {
    await db.delete("auth", "current");
    return undefined;
  }
  return record;
}

export async function clearAuthState(): Promise<void> {
  const db = await getDb();
  await db.delete("auth", "current");
}
