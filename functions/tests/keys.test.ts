import request from "supertest";
import jwt from "jsonwebtoken";
import { app } from "../src/index";

const JWT_SECRET = "test-secret-min-32-chars-long!!";

jest.mock("../src/utils/secrets", () => ({
  getJwtSecret: jest.fn().mockResolvedValue("test-secret-min-32-chars-long!!"),
  getVapidKeys: jest.fn().mockResolvedValue({
    publicKey: "BEl62iUYgUivxIkv69yViEuiBIa-Ib9-SkvMeAtA3LFgDzkrxZJjSgSnfckjBJuBkr3qBUYIHBQFLXYp5Nksh8U",
    privateKey: "UUxI4O8-FbRouAevSmBQ6co550gBRqK-hLRJEJMGbUA",
  }),
}));

// Mock Firestore
const mockOneTimePreKeys: Array<{ id: number; publicKey: string }> = [];
const mockBundleData = {
  identityPublicKey: "test-identity-key",
  signingPublicKey: "test-signing-key",
  signedPreKey: { id: 1, publicKey: "test-spk", signature: "test-sig" },
  get oneTimePreKeys() {
    return mockOneTimePreKeys;
  },
};

const mockRunTransaction = jest.fn(async (cb: (tx: unknown) => Promise<void>) => {
  const tx = {
    get: jest.fn().mockResolvedValue({ exists: true, data: () => mockBundleData }),
    set: jest.fn(),
    update: jest.fn(),
  };
  await cb(tx);
});

const mockBundleSnap = {
  exists: true,
  data: () => mockBundleData,
};

const mockBundleRef = {
  set: jest.fn().mockResolvedValue(undefined),
  get: jest.fn().mockResolvedValue(mockBundleSnap),
  update: jest.fn().mockResolvedValue(undefined),
};

jest.mock("../src/utils/firestore", () => ({
  getDb: jest.fn().mockReturnValue({
    collection: jest.fn().mockReturnValue({
      doc: jest.fn().mockReturnValue(mockBundleRef),
    }),
    runTransaction: mockRunTransaction,
  }),
}));

function makeAuthHeader(uid: string): string {
  const token = jwt.sign({ uid }, JWT_SECRET, { expiresIn: "1h" });
  return `Bearer ${token}`;
}

describe("POST /keys/identity", () => {
  it("returns 401 without auth", async () => {
    const res = await request(app).post("/keys/identity").send({});
    expect(res.status).toBe(401);
  });

  it("returns 400 if keys are missing", async () => {
    const res = await request(app)
      .post("/keys/identity")
      .set("Authorization", makeAuthHeader("user-1"))
      .send({});
    expect(res.status).toBe(400);
  });

  it("stores identity keys and returns ok", async () => {
    const res = await request(app)
      .post("/keys/identity")
      .set("Authorization", makeAuthHeader("user-1"))
      .send({ identityPublicKey: "abc", signingPublicKey: "xyz" });
    expect(res.status).toBe(200);
    expect(res.body.ok).toBe(true);
  });
});

describe("GET /keys/bundle", () => {
  it("returns 400 without uid param", async () => {
    const res = await request(app)
      .get("/keys/bundle")
      .set("Authorization", makeAuthHeader("user-1"));
    expect(res.status).toBe(400);
  });

  it("returns countOnly when requested", async () => {
    mockOneTimePreKeys.length = 0;
    mockOneTimePreKeys.push({ id: 1, publicKey: "k1" }, { id: 2, publicKey: "k2" });
    const res = await request(app)
      .get("/keys/bundle?uid=user-2&countOnly=true")
      .set("Authorization", makeAuthHeader("user-1"));
    expect(res.status).toBe(200);
    expect(res.body.oneTimePreKeyCount).toBe(2);
  });

  it("returns full bundle and pops a one-time prekey", async () => {
    mockOneTimePreKeys.length = 0;
    mockOneTimePreKeys.push({ id: 10, publicKey: "opk-10" });
    const res = await request(app)
      .get("/keys/bundle?uid=user-2")
      .set("Authorization", makeAuthHeader("user-1"));
    expect(res.status).toBe(200);
    expect(res.body.identityPublicKey).toBeDefined();
    expect(res.body.signedPreKey).toBeDefined();
    expect(res.body.oneTimePreKey).toBeDefined();
  });
});
