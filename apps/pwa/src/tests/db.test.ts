/**
 * Unit tests for the IndexedDB persistence layer.
 *
 * vitest uses jsdom which provides a partial IndexedDB implementation.
 * We use the fake-indexeddb library approach via idb's openDB.
 */

import { describe, it, expect, beforeEach, vi } from "vitest";

// Mock idb with an in-memory store
vi.mock("idb", () => {
  const stores: Record<string, Map<unknown, unknown>> = {};

  function getStore(name: string) {
    if (!stores[name]) stores[name] = new Map();
    return stores[name];
  }

  const mockDb = {
    put: vi.fn(async (storeName: string, value: Record<string, unknown>) => {
      const store = getStore(storeName);
      const keyPath = storeName === "auth" ? "id"
        : storeName === "identity" ? "uid"
        : storeName === "sessions" ? "roomId"
        : "id";
      store.set(value[keyPath], value);
    }),
    get: vi.fn(async (storeName: string, key: unknown) => {
      return getStore(storeName).get(key);
    }),
    getAll: vi.fn(async (storeName: string) => {
      return Array.from(getStore(storeName).values());
    }),
    count: vi.fn(async (storeName: string) => {
      return getStore(storeName).size;
    }),
    delete: vi.fn(async (storeName: string, key: unknown) => {
      getStore(storeName).delete(key);
    }),
    transaction: vi.fn((storeName: string, _mode: string) => ({
      store: {
        put: vi.fn(async (value: Record<string, unknown>) => {
          getStore(storeName).set(value.id, value);
        }),
      },
      done: Promise.resolve(),
    })),
  };

  return {
    openDB: vi.fn().mockResolvedValue(mockDb),
  };
});

import {
  saveIdentity,
  loadIdentity,
  saveSignedPreKey,
  loadLatestSignedPreKey,
  saveOneTimePreKeys,
  countOneTimePreKeys,
  getNextOneTimePreKeyId,
  saveAuthState,
  loadAuthState,
  clearAuthState,
} from "../store/db";

beforeEach(() => {
  vi.clearAllMocks();
});

describe("identity store", () => {
  it("saves and loads identity", async () => {
    const identity = {
      uid: "user-1",
      identityPrivateKey: "priv",
      identityPublicKey: "pub",
      signingPrivateKey: "spriv",
      signingPublicKey: "spub",
      createdAt: 1000,
    };
    await saveIdentity(identity);
    const loaded = await loadIdentity("user-1");
    expect(loaded?.uid).toBe("user-1");
    expect(loaded?.identityPublicKey).toBe("pub");
  });

  it("returns undefined for unknown uid", async () => {
    const result = await loadIdentity("nonexistent");
    expect(result).toBeUndefined();
  });
});

describe("signed prekey store", () => {
  it("saves and loads the latest signed prekey", async () => {
    await saveSignedPreKey({ id: 1, publicKey: "pk1", privateKey: "sk1", signature: "sig1", createdAt: 1000 });
    await saveSignedPreKey({ id: 2, publicKey: "pk2", privateKey: "sk2", signature: "sig2", createdAt: 2000 });
    const latest = await loadLatestSignedPreKey();
    expect(latest?.id).toBe(2);
  });
});

describe("one-time prekey store", () => {
  it("saves multiple one-time prekeys and counts them", async () => {
    await saveOneTimePreKeys([
      { id: 1, publicKey: "pk1", privateKey: "sk1" },
      { id: 2, publicKey: "pk2", privateKey: "sk2" },
    ]);
    const count = await countOneTimePreKeys();
    expect(count).toBe(2);
  });

  it("computes next OPK id correctly", async () => {
    await saveOneTimePreKeys([{ id: 5, publicKey: "pk5", privateKey: "sk5" }]);
    const nextId = await getNextOneTimePreKeyId();
    expect(nextId).toBe(6);
  });
});

describe("auth state store", () => {
  it("saves and loads auth state", async () => {
    await saveAuthState({ uid: "u1", jwt: "tok", expiresAt: Date.now() + 3600_000 });
    const loaded = await loadAuthState();
    expect(loaded?.uid).toBe("u1");
    expect(loaded?.jwt).toBe("tok");
  });

  it("returns undefined for expired auth", async () => {
    await saveAuthState({ uid: "u2", jwt: "old", expiresAt: Date.now() - 1 });
    const loaded = await loadAuthState();
    expect(loaded).toBeUndefined();
  });

  it("clears auth state", async () => {
    await saveAuthState({ uid: "u3", jwt: "tok3", expiresAt: Date.now() + 3600_000 });
    await clearAuthState();
    const loaded = await loadAuthState();
    expect(loaded).toBeUndefined();
  });
});
